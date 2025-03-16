import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

interface AppInfo {
  name: string;
  path: string;
  icon_path: string | null;
  description: string | null;
}

function App() {
  const [query, setQuery] = useState('');
  const [apps, setApps] = useState<AppInfo[]>([]);
  const [results, setResults] = useState<AppInfo[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isLoading, setIsLoading] = useState(true);
  const resultsContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    async function loadApps() {
      try {
        setIsLoading(true);
        const installedApps = await invoke<AppInfo[]>('get_installed_apps');
        setApps(installedApps);
        setIsLoading(false);
      } catch (error) {
        console.error('Failed to load apps:', error);
        setIsLoading(false);
      }
    }

    loadApps();
  }, []);

  useEffect(() => {
    async function performSearch() {
      if (!query.trim()) {
        setResults([]);
        return;
      }

      try {
        const searchResults = await invoke<AppInfo[]>('search_applications', {
          query,
        });
        setResults(searchResults);
        setSelectedIndex(0);
      } catch (error) {
        console.error('Search failed:', error);
      }
    }

    performSearch();
  }, [query]);

  useEffect(() => {
    if (resultsContainerRef.current) {
      const searchHeight = 60;
      const resultsHeight =
        results.length > 0 ? Math.min(400, results.length * 44 + 16) : 0;

      const totalHeight =
        searchHeight + (resultsHeight > 0 ? resultsHeight : 0);

      invoke('resize_window', { height: totalHeight });
    }
  }, [results]);

  const launchSelectedApp = async () => {
    if (results[selectedIndex]) {
      try {
        await invoke('launch_app', { appPath: results[selectedIndex].path });
        setQuery(''); // Clear search after launching
      } catch (error) {
        console.error('Failed to launch app:', error);
      }
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev < results.length - 1 ? prev + 1 : prev
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((prev) => (prev > 0 ? prev - 1 : prev));
        break;
      case 'Enter':
        e.preventDefault();
        launchSelectedApp();
        break;
      case 'Escape':
        setQuery('');
        break;
    }
  };

  return (
    <div className='raycast-wrapper'>
      <div
        className={`raycast-container ${
          results.length > 0 ? 'has-results' : ''
        }`}
      >
        <div className='search-container'>
          <div className='search-icon'>⌘</div>
          <input
            id='search-input'
            type='text'
            className='search-input'
            placeholder={
              isLoading ? 'Loading applications...' : 'Search applications...'
            }
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            autoFocus
            disabled={isLoading}
          />
          {query && (
            <button
              className='clear-button'
              onClick={() => setQuery('')}
            >
              ×
            </button>
          )}
        </div>

        {results.length > 0 && <div className='results-divider'></div>}

        <div
          ref={resultsContainerRef}
          className={`results-container ${results.length > 0 ? 'visible' : ''}`}
        >
          {isLoading ? (
            <div className='loading'>Loading applications...</div>
          ) : results.length > 0 ? (
            <ul className='results-list'>
              {results.map((app, index) => (
                <li
                  key={index}
                  className={`result-item ${
                    selectedIndex === index ? 'selected' : ''
                  }`}
                  onClick={() => {
                    setSelectedIndex(index);
                    launchSelectedApp();
                  }}
                >
                  {app.name}
                  <span className='app-path'>{app.path}</span>
                </li>
              ))}
            </ul>
          ) : query.trim() !== '' ? (
            <div className='no-results'>No results found</div>
          ) : null}
        </div>
      </div>
    </div>
  );
}

export default App;
