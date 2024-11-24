// backend/server.js

const express = require('express');
const cors = require('cors');
const path = require('path');

const app = express();
const PORT = 5000; // Changed to 5000 to avoid conflict with React's default port 3000

// Middleware
app.use(cors({
  origin: 'http://localhost:3000', // Allow React app to communicate
}));
app.use(express.json());
app.use(express.static(path.join(__dirname, 'public')));

// In-memory storage for retrospectives
let retros = [];

/**
 * Utility function to find a retro by ID.
 * @param {number} id - The ID of the retro.
 * @returns {object|null} - The retro object or null if not found.
 */
function findRetroById(id) {
  return retros.find(retro => retro.id === id) || null;
}

// API Endpoints

/**
 * GET /api/retros
 * Returns the list of all retrospectives.
 */
app.get('/api/retros', (req, res) => {
  res.json(retros);
});

/**
 * POST /api/retros
 * Creates a new retrospective.
 * Expects { retroName: String, creatorName: String } in the request body.
 */
app.post('/api/retros', (req, res) => {
  const { retroName, creatorName } = req.body;

  // Validate input
  if (!retroName || !creatorName) {
    return res.status(400).json({ message: 'retroName and creatorName are required.' });
  }

  // Create a new retro object
  const newRetro = {
    id: retros.length + 1,
    retroName,
    creatorName,
    createdAt: new Date(),
    users: [], // List of usernames
    cards: {
      good: [],              // Array of card texts
      bad: [],
      needsImprovement: []
    }
  };

  // Add to retrospectives list
  retros.push(newRetro);

  // Respond with the new retro
  res.status(201).json(newRetro);
});

/**
 * POST /api/retros/:id/users
 * Adds a user to a specific retro.
 * Expects { username: String } in the request body.
 */
app.post('/api/retros/:id/users', (req, res) => {
  const retroId = parseInt(req.params.id, 10);
  const { username } = req.body;

  if (!username) {
    return res.status(400).json({ message: 'Username is required.' });
  }

  const retro = findRetroById(retroId);
  if (!retro) {
    return res.status(404).json({ message: 'Retro not found.' });
  }

  // Add user to retro if not already present
  if (!retro.users.includes(username)) {
    retro.users.push(username);
  }

  res.status(200).json({ users: retro.users });
});

/**
 * GET /api/retros/:id/users
 * Retrieves the list of users in a specific retro.
 */
app.get('/api/retros/:id/users', (req, res) => {
  const retroId = parseInt(req.params.id, 10);
  const retro = findRetroById(retroId);

  if (!retro) {
    return res.status(404).json({ message: 'Retro not found.' });
  }

  res.json({ users: retro.users });
});

/**
 * GET /api/retros/:id/cards
 * Retrieves all cards in a specific retro, categorized by type.
 */
app.get('/api/retros/:id/cards', (req, res) => {
  const retroId = parseInt(req.params.id, 10);
  const retro = findRetroById(retroId);

  if (!retro) {
    return res.status(404).json({ message: 'Retro not found.' });
  }

  res.json({ cards: retro.cards });
});

/**
 * POST /api/retros/:id/cards
 * Adds a card to a specific column in the retro.
 * Expects { category: 'good' | 'bad' | 'needsImprovement', text: String } in the request body.
 */
app.post('/api/retros/:id/cards', (req, res) => {
  const retroId = parseInt(req.params.id, 10);
  const { category, text } = req.body;

  if (!category || !text) {
    return res.status(400).json({ message: 'Category and text are required.' });
  }

  const validCategories = ['good', 'bad', 'needsImprovement'];
  if (!validCategories.includes(category)) {
    return res.status(400).json({ message: 'Invalid category.' });
  }

  const retro = findRetroById(retroId);
  if (!retro) {
    return res.status(404).json({ message: 'Retro not found.' });
  }

  // Add the card to the specified category
  retro.cards[category].push(text);

  res.status(201).json({ category, text });
});

/**
 * Fallback route to serve index.html for any undefined routes except /retros/:id
 */
app.get('*', (req, res) => {
  const urlPath = req.path;
  const retroPathRegex = /^\/retros\/\d+$/;

  if (retroPathRegex.test(urlPath)) {
    res.sendFile(path.join(__dirname, 'public', 'retro.html'));
  } else {
    res.sendFile(path.join(__dirname, 'public', 'index.html'));
  }
});

// Start the server
app.listen(PORT, () => {
  console.log(`Server is running on http://localhost:${PORT}`);
});
