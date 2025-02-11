// frontend/src/components/HomePage.js

import React, { useState, useEffect } from 'react';
import { Typography, Box } from '@mui/material';
import LoginForm from './LoginForm';
import RetroList from './RetroList';

const HomePage = () => {
  const [username, setUsername] = useState('');

  useEffect(() => {
    const savedUsername = sessionStorage.getItem('username');
    if (savedUsername) {
      setUsername(savedUsername);
    }
  }, []);

  const handleSaveUsername = (name) => {
    setUsername(name);
    sessionStorage.setItem('username', name);
  };

  return (
    <Box textAlign="center" mt={4}>
      <Typography variant="h3" gutterBottom>
        Team Retrospectives
      </Typography>

      {!username ? (
        <LoginForm onSave={handleSaveUsername} />
      ) : (
        <Box mt={4}>
          <Typography variant="h6">Welcome, {username}!</Typography>
          <RetroList username={username} />
        </Box>
      )}
    </Box>
  );
};

export default HomePage;
