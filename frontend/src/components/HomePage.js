// frontend/src/components/HomePage.js

import React, { useState, useEffect } from 'react';
import { Typography, Box } from '@mui/material';
import UsernameForm from './UsernameForm';
import RetroList from './RetroList';

const HomePage = () => {
  const [username, setUsername] = useState('');
  const [user_id, setUserId] = useState('');

  useEffect(() => {
    const savedUsername = sessionStorage.getItem('username');
    const savedUserId = sessionStorage.getItem('userid');
    if (savedUsername) {
      setUsername(savedUsername);
      setUserId(savedUserId)
    }
  }, []);

  const handleSaveUsername = (name, id) => {
    setUsername(name);
    setUserId(id)
  };

  return (
    <Box textAlign="center" mt={4}>
      <Typography variant="h3" gutterBottom>
        Team Retrospectives
      </Typography>

      {!username ? (
        <UsernameForm onSave={handleSaveUsername} />
      ) : (
        <Box mt={4}>
          <Typography variant="h6">Welcome, {username}!</Typography>
          <RetroList username={username} user_id={user_id} />
        </Box>
      )}
    </Box>
  );
};

export default HomePage;
