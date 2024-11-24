// frontend/src/components/RetroPage.js

import React, { useEffect, useState } from 'react';
import {
  Typography,
  Box,
  Grid,
  Paper,
  TextField,
  Button,
  List,
  ListItem,
  ListItemText,
} from '@mui/material';
import { useParams, useNavigate } from 'react-router-dom';
import Sidebar from './Sidebar';
import Column from './Column';

const RetroPage = () => {
  const { id } = useParams();
  const navigate = useNavigate();

  const [retroName, setRetroName] = useState('');
  const [users, setUsers] = useState([]);
  const [cards, setCards] = useState({
    good: [],
    bad: [],
    needsImprovement: [],
  });
  const [newCards, setNewCards] = useState({
    good: '',
    bad: '',
    needsImprovement: '',
  });

  const username = sessionStorage.getItem('username');

  useEffect(() => {
    if (!username) {
      alert('Username not found. Please set your username on the homepage.');
      navigate('/');
    }

    // Add user to retro
    fetch(`/api/retros/${id}/users`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username }),
    })
      .then((res) => res.json())
      .then((data) => setUsers(data.users))
      .catch((err) => console.error('Error adding user to retro:', err));

    // Fetch retro details
    fetch(`/api/retros`)
      .then((res) => res.json())
      .then((data) => {
        const retro = data.find((r) => r.id === parseInt(id, 10));
        if (retro) {
          setRetroName(retro.retroName);
          setCards(retro.cards);
        } else {
          alert('Retro not found.');
          navigate('/');
        }
      })
      .catch((err) => console.error('Error fetching retro details:', err));

    // Optionally, set up real-time updates with WebSockets or polling
    const interval = setInterval(() => {
      fetch(`/api/retros/${id}/users`)
        .then((res) => res.json())
        .then((data) => setUsers(data.users))
        .catch((err) => console.error('Error fetching users:', err));

      fetch(`/api/retros/${id}/cards`)
        .then((res) => res.json())
        .then((data) => setCards(data.cards))
        .catch((err) => console.error('Error fetching cards:', err));
    }, 5000); // Update every 5 seconds

    return () => clearInterval(interval);
  }, [id, username, navigate]);

  const handleAddCard = (category) => {
    const text = newCards[category].trim();
    if (!text) {
      alert('Please enter some text for the card.');
      return;
    }

    fetch(`/api/retros/${id}/cards`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ category, text }),
    })
      .then((res) => res.json())
      .then((data) => {
        setCards((prev) => ({
          ...prev,
          [category]: [...prev[category], data.text],
        }));
        setNewCards((prev) => ({ ...prev, [category]: '' }));
      })
      .catch((err) => {
        console.error('Error adding card:', err);
        alert('Failed to add card.');
      });
  };

  const handleLeaveRetro = () => {
    sessionStorage.removeItem('username');
    navigate('/');
  };

  return (
    <Box display="flex" height="100vh">
      <Sidebar users={users} />

      <Box flexGrow={1} p={4} overflow="auto">
        <Box display="flex" justifyContent="space-between" alignItems="center" mb={4}>
          <Typography variant="h4">{retroName}</Typography>
          <Box display="flex" alignItems="center">
            <Typography variant="subtitle1" mr={2}>
              Welcome, {username}!
            </Typography>
            <Button variant="contained" color="secondary" onClick={handleLeaveRetro}>
              Leave Retro
            </Button>
          </Box>
        </Box>

        <Grid container spacing={4}>
          <Grid item xs={12} md={4}>
            <Column
              title="Good"
              cards={cards.good}
              newCardText={newCards.good}
              onNewCardTextChange={(text) => setNewCards((prev) => ({ ...prev, good: text }))}
              onAddCard={() => handleAddCard('good')}
            />
          </Grid>
          <Grid item xs={12} md={4}>
            <Column
              title="Bad"
              cards={cards.bad}
              newCardText={newCards.bad}
              onNewCardTextChange={(text) => setNewCards((prev) => ({ ...prev, bad: text }))}
              onAddCard={() => handleAddCard('bad')}
            />
          </Grid>
          <Grid item xs={12} md={4}>
            <Column
              title="Needs Improvement"
              cards={cards.needsImprovement}
              newCardText={newCards.needsImprovement}
              onNewCardTextChange={(text) => setNewCards((prev) => ({ ...prev, needsImprovement: text }))}
              onAddCard={() => handleAddCard('needsImprovement')}
            />
          </Grid>
        </Grid>
      </Box>
    </Box>
  );
};

export default RetroPage;
