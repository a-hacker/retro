// frontend/src/App.js

import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { CssBaseline, Container } from '@mui/material';
import HomePage from './components/HomePage';
import RetroPage from './components/RetroPage';
import ApolloProvider from './ApolloProvider'; // Import the ApolloProvider

function App() {
  return (
    <ApolloProvider>
      <Router>
        <CssBaseline />
        <Container maxWidth="xl" disableGutters>
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/retros/:id" element={<RetroPage />} />
          </Routes>
        </Container>
      </Router>
    </ApolloProvider>
  );
}

export default App;
