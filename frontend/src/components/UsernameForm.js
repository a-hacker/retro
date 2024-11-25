// frontend/src/components/UsernameForm.js

import React, { useState } from 'react';
import { TextField, Button, Box } from '@mui/material';

const UsernameForm = ({ onSave }) => {
  const [username, setUsername] = useState('');

  const handleSave = () => {
    const trimmedUsername = username.trim();
    if (trimmedUsername) {
      sessionStorage.setItem('username', trimmedUsername);
      onSave(trimmedUsername);
    } else {
      alert('Please enter a valid username.');
    }
  };

  const handleKeyPress = (e) => {
    if (e.key === 'Enter') {
      handleSave();
    }
  };

  return (
    <Box display="flex" flexDirection="column" alignItems="center" mt={4}>
      <TextField
        label="Enter your username"
        variant="outlined"
        value={username}
        onChange={(e) => setUsername(e.target.value)}
        onKeyPress={handleKeyPress}
        fullWidth
        sx={{ maxWidth: 400 }}
      />
      <Button
        variant="contained"
        color="primary"
        onClick={handleSave}
        sx={{ mt: 2 }}
      >
        Save
      </Button>
    </Box>
  );
};

export default UsernameForm;
