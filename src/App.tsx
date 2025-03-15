import { useState, useEffect } from 'react';
import './App.css';

function App() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<string[]>([]);

  const dummyItems = [
    'Visual Studio Code',
    'Chrome',
    'Firefox',
    'Notepad',
    'Calculator',
    'Settings',
    'Explorer',
    'Terminal',
    'Spotify',
    'Discord',
    'Slack',
  ];

  useEffect(() => {
    if (query.trim() === '') {
      setResults([]);
    } else {
      const filtered = dummyItems.filter((item) =>
        item.toLowerCase().includes(query.toLowerCase())
      );
      setResults(filtered);
    }
  }, [query]);

  useEffect(() => {
    const searchInput = document.getElementById('search-input');
    if (searchInput) {
      searchInput.focus();
    }
  }, []);

  return (
    <div className='app-container'>
      <div className='search-container'>
        <input
          id='search-input'
          type='text'
          className='search-input'
          placeholder='Search for apps, files, web...'
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          autoFocus
        />

        <div className='results-container'>
          {results.length > 0 ? (
            <ul className='results-list'>
              {results.map((item, index) => (
                <li
                  key={index}
                  className='result-item'
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

      <div className='info-text'>Press Alt+Space to toggle this window</div>
    </div>
  );
}

export default App;
