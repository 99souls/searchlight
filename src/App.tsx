import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useVirtualizer } from '@tanstack/react-virtual';
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

  const virtualizer = useVirtualizer({
    count: results.length,
    getScrollElement: () => resultsContainerRef.current,
    estimateSize: () => 44,
    overscan: 5,
    paddingStart: 8,
    paddingEnd: 8,
  });

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

  const getIconUrl = (iconPath: string | undefined) => {
    if (!iconPath) return undefined;
    try {
      return convertFileSrc(iconPath);
    } catch (e) {
      console.error('Failed to convert icon path', e);
      return undefined;
    }
  };

  useEffect(() => {
    if (!query.trim()) {
      setResults([]);
      return;
    }

    const searchResults = apps.filter(
      (app) =>
        app.name.toLowerCase().includes(query.toLowerCase()) ||
        (app.description &&
          app.description.toLowerCase().includes(query.toLowerCase()))
    );

    if (results.length !== searchResults.length) {
      setSelectedIndex(0);
    }

    setResults(searchResults);
  }, [query, apps]);

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

  useEffect(() => {
    if (results.length > 0) {
      virtualizer.scrollToIndex(selectedIndex, { align: 'auto' });
    }
  }, [selectedIndex, virtualizer, results.length]);

  const launchSelectedApp = async () => {
    if (results.length > 0 && results[selectedIndex]) {
      try {
        await invoke('launch_app', { appPath: results[selectedIndex].path });
        setQuery('');
      } catch (error) {
        console.error('Failed to launch app:', error);
      }
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (results.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((prev) => (prev < results.length - 1 ? prev + 1 : 0));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((prev) => (prev > 0 ? prev - 1 : results.length - 1));
        break;
      case 'Enter':
        e.preventDefault();
        launchSelectedApp();
        break;
      case 'Escape':
        e.preventDefault();
        setQuery('');
        break;
    }
  };

  return (
    <div className='searchlight-wrapper'>
      <div
        className={`searchlight-container ${
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
          style={{
            height:
              results.length > 0 ? Math.min(400, results.length * 44 + 16) : 0,
          }}
        >
          {isLoading ? (
            <div className='loading'>Loading applications...</div>
          ) : results.length > 0 ? (
            <div
              className='results-list-virtual'
              style={{
                height: `${virtualizer.getTotalSize()}px`,
                width: '100%',
                position: 'relative',
              }}
            >
              {virtualizer.getVirtualItems().map((virtualItem) => {
                const app = results[virtualItem.index];

                return (
                  <div
                    key={virtualItem.key}
                    className={`result-item ${
                      selectedIndex === virtualItem.index ? 'selected' : ''
                    }`}
                    style={{
                      position: 'absolute',
                      top: 0,
                      left: 0,
                      width: 'calc(100% - 16px)',
                      height: `${virtualItem.size}px`,
                      transform: `translateY(${virtualItem.start}px)`,
                    }}
                    onClick={() => {
                      setSelectedIndex(virtualItem.index);
                      launchSelectedApp();
                    }}
                  >
                    <div className='app-icon-container'>
                      {app.icon_path ? (
                        <img
                          src={getIconUrl(app.icon_path)}
                          className='app-icon'
                          alt=''
                        />
                      ) : (
                        <div className='app-icon-placeholder'>📱</div>
                      )}
                    </div>
                    <span className='app-name'>{app.name}</span>
                  </div>
                );
              })}
            </div>
          ) : query.trim() !== '' ? (
            <div className='no-results'>No results found</div>
          ) : null}
        </div>
      </div>
    </div>
  );
}

export default App;
