// backend/server.js

const express = require('express');
const cors = require('cors');
const path = require('path');
const { Server } = require('socket.io');
const http = require('http');

const app = express();
const PORT = 5000; // Backend runs on port 5000

// Create HTTP server
const server = http.createServer(app);

// Initialize Socket.io
const io = new Server(server, {
  cors: {
    origin: 'http://localhost:3000', // React app runs on port 3000
    methods: ['GET', 'POST', 'DELETE'],
  },
});

// Middleware
app.use(cors({
  origin: 'http://localhost:3000', // Allow React app to communicate
  methods: ['GET', 'POST', 'DELETE'],
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

// Socket.io connection handling
io.on('connection', (socket) => {
  console.log('A user connected:', socket.id);

  // Join a retro room
  socket.on('joinRetro', (retroId, username) => {
    socket.join(`retro_${retroId}`);
    const retro = findRetroById(retroId);
    retro.users.push(username);
    console.log(`${username} joined retro ${retroId}`);
  });

  // Handle adding a new card
  socket.on('addCard', (retroId, category, text) => {
    const retro = findRetroById(retroId);
    if (retro && retro.cards[category]) {
      retro.cards[category].push(text);
      // Emit to all clients in the retro room
      io.to(`retro_${retroId}`).emit('newCard', category, text);
    }
  });

  // Handle user disconnection (Optional: Remove user from all retros)
  socket.on('disconnect', () => {
    console.log('User disconnected:', socket.id);
    // Implement user removal logic if tracking socket-user associations
  });
});

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
    // Emit updated users list to all clients in the retro room
    io.to(`retro_${retroId}`).emit('updateUsers', retro.users);
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

  // Emit new card to retro room
  io.to(`retro_${retroId}`).emit('newCard', category, text);

  res.status(201).json({ category, text });
});

/**
 * DELETE /api/retros/:id/users
 * Removes a user from a specific retro.
 * Expects { username: String } in the request body.
 */
app.delete('/api/retros/:id/users', (req, res) => {
  const retroId = parseInt(req.params.id, 10);
  const { username } = req.body;

  if (!username) {
    return res.status(400).json({ message: 'Username is required.' });
  }

  const retro = findRetroById(retroId);
  if (!retro) {
    return res.status(404).json({ message: 'Retro not found.' });
  }

  const userIndex = retro.users.indexOf(username);
  if (userIndex > -1) {
    retro.users.splice(userIndex, 1);

    // Emit updated users list to all clients in the retro room
    io.to(`retro_${retroId}`).emit('updateUsers', retro.users);

    res.status(200).json({ users: retro.users });
  } else {
    res.status(404).json({ message: 'User not found in this retro.' });
  }
});

/**
 * Fallback route to serve React's index.html for any non-API routes.
 * This allows React Router to handle client-side routing.
 */
app.get('*', (req, res) => {
  res.sendFile(path.join(__dirname, 'public', 'index.html'));
});

// Start the server with Socket.io
server.listen(PORT, () => {
  console.log(`Server is running on http://localhost:${PORT}`);
});
