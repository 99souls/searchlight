import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

function App() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<string[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const resultsContainerRef = useRef<HTMLDivElement>(null);

  // Dummy data
  const dummyItems = [
    { name: 'Visual Studio Code', icon: '💻' },
    { name: 'Chrome', icon: '🌐' },
    { name: 'Firefox', icon: '🦊' },
    { name: 'Slack', icon: '💬' },
    { name: 'Spotify', icon: '🎵' },
    { name: 'Terminal', icon: '📟' },
    { name: 'Notes', icon: '📝' },
    { name: 'Calculator', icon: '🧮' },
    { name: 'Settings', icon: '⚙️' },
    { name: 'Mail', icon: '📧' },
  ];

  useEffect(() => {
    if (query.trim() === '') {
      setResults([]);
    } else {
      const filteredItems = dummyItems
        .filter((item) => item.name.toLowerCase().includes(query.toLowerCase()))
        .map((item) => `${item.icon} ${item.name}`);
      setResults(filteredItems);
      setSelectedIndex(0);
    }
  }, [query]);

  useEffect(() => {
    const searchInput = document.getElementById('search-input');
    if (searchInput) {
      searchInput.focus();
    }
  }, []);

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
        if (results[selectedIndex]) {
          console.log(`Launching: ${results[selectedIndex]}`);
        }
        break;
      case 'Escape':
        setQuery('');
        break;
    }
  };

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
            placeholder='Search applications...'
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            autoFocus
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
          {results.length > 0 ? (
            <ul className='results-list'>
              {results.map((item, index) => (
                <li
                  key={index}
                  className={`result-item ${
                    selectedIndex === index ? 'selected' : ''
                  }`}
                  onClick={() => console.log(`Clicked: ${item}`)}
                >
                  {item}
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
