// frontend/src/components/RetroList.js

import React, { useEffect, useState } from 'react';
import {
  List,
  ListItem,
  ListItemText,
  Button,
  TextField,
  Box,
  Typography,
} from '@mui/material';
import { Link } from 'react-router-dom';

const RetroList = ({ username }) => {
  const [retros, setRetros] = useState([]);
  const [newRetroName, setNewRetroName] = useState('');

  const fetchRetros = () => {
    fetch('/api/retros')
      .then((res) => res.json())
      .then((data) => setRetros(data))
      .catch((err) => console.error('Error fetching retros:', err));
  };

  useEffect(() => {
    fetchRetros();
    // Optionally, set up polling or WebSockets for real-time updates
  }, []);

  const handleCreateRetro = () => {
    const trimmedName = newRetroName.trim();
    if (!trimmedName) {
      alert('Please enter a retro name.');
      return;
    }

    fetch('/api/retros', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ retroName: trimmedName, creatorName: username }),
    })
      .then((res) => {
        if (!res.ok) {
          return res.json().then((err) => { throw err; });
        }
        return res.json();
      })
      .then((newRetro) => {
        setNewRetroName('');
        setRetros([...retros, newRetro]);
      })
      .catch((err) => {
        console.error('Error creating retro:', err);
        alert(err.message || 'Failed to create retro.');
      });
  };

  return (
    <Box mt={4}>
      <Typography variant="h5" gutterBottom>
        Available Retrospectives
      </Typography>
      <List>
        {retros.length === 0 ? (
          <Typography>No retrospectives available. Create one!</Typography>
        ) : (
          retros.map((retro) => (
            <ListItem
              key={retro.id}
              button
              component={Link}
              to={`/retros/${retro.id}`}
            >
              <ListItemText
                primary={retro.retroName}
                secondary={`Created by: ${retro.creatorName}`}
              />
            </ListItem>
          ))
        )}
      </List>

      <Box mt={4}>
        <Typography variant="h5" gutterBottom>
          Create a New Retro
        </Typography>
        <Box display="flex" alignItems="center">
          <TextField
            label="Retro Name"
            variant="outlined"
            value={newRetroName}
            onChange={(e) => setNewRetroName(e.target.value)}
            fullWidth
            sx={{ mr: 2 }}
          />
          <Button variant="contained" color="primary" onClick={handleCreateRetro}>
            Create Retro
          </Button>
        </Box>
      </Box>
    </Box>
  );
};

export default RetroList;
