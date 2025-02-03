// frontend/src/App.js

import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { CssBaseline, Container } from '@mui/material';
import HomePage from './components/HomePage';
import RetroPage from './components/RetroPage';
import ApolloProvider from './ApolloProvider';
import { SnackbarProvider } from 'notistack';

function App() {
  return (
    <ApolloProvider>
      <SnackbarProvider maxSnack={3}>
        <Router>
          <CssBaseline />
          <Container maxWidth="100%" disableGutters>
            <Routes>
              <Route path="/retro" element={<HomePage />} />
              <Route path="/retro/:id" element={<RetroPage />} />
            </Routes>
          </Container>
        </Router>
      </SnackbarProvider>
    </ApolloProvider>
  );
}

export default App;
