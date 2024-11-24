// frontend/src/App.js

import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Container, CssBaseline } from '@mui/material';
import HomePage from './components/HomePage';
import RetroPage from './components/RetroPage';

function App() {
  return (
    <Router>
      <CssBaseline />
      <Container maxWidth="lg">
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/retros/:id" element={<RetroPage />} />
        </Routes>
      </Container>
    </Router>
  );
}

export default App;
