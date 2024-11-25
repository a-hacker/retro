// frontend/src/components/RetroPage.js

import React, { useEffect, useState } from 'react';
import {
  Typography,
  Box,
  Grid,
  Button,
} from '@mui/material';
import { useParams, useNavigate } from 'react-router-dom';
import Sidebar from './Sidebar';
import Column from './Column';
import { io } from 'socket.io-client';

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
  const [socket, setSocket] = useState(null);

  useEffect(() => {
    if (!username) {
      alert('Username not found. Please set your username on the homepage.');
      navigate('/');
      return;
    }

    // Initialize Socket.io client
    const newSocket = io('http://localhost:5000'); // Adjust if backend is hosted elsewhere
    setSocket(newSocket);

    // On connection, join the retro room
    newSocket.on('connect', () => {
      newSocket.emit('joinRetro', parseInt(id, 10), username);
    });

    // Listen for new cards
    newSocket.on('newCard', (category, text) => {
      setCards((prev) => ({
        ...prev,
        [category]: [...prev[category], text],
      }));
    });

    // Listen for user updates
    newSocket.on('updateUsers', (updatedUsers) => {
      setUsers(updatedUsers);
    });

    // Cleanup on unmount
    return () => {
      newSocket.disconnect();
    };
  }, [id, username, navigate]);

  useEffect(() => {
    // Fetch retro details
    fetch(`/api/retros`)
      .then((res) => res.json())
      .then((data) => {
        const retro = data.find((r) => r.id === parseInt(id, 10));
        if (retro) {
          setRetroName(retro.retroName);
          setCards(retro.cards);
          setUsers(retro.users);
        } else {
          alert('Retro not found.');
          navigate('/');
        }
      })
      .catch((err) => console.error('Error fetching retro details:', err));
  }, [id, navigate]);

  useEffect(() => {
    // Optional: Fetch initial cards if not already set
    if (cards.good.length === 0 && cards.bad.length === 0 && cards.needsImprovement.length === 0) {
      fetch(`/api/retros/${id}/cards`)
        .then((res) => res.json())
        .then((data) => setCards(data.cards))
        .catch((err) => console.error('Error fetching cards:', err));
    }
  }, [id, cards]);

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
      .then((res) => {
        if (!res.ok) {
          return res.json().then((err) => { throw err; });
        }
        return res.json();
      })
      .then((data) => {
        setNewCards((prev) => ({ ...prev, [category]: '' }));
        // The new card will be added via the 'newCard' Socket.io event
      })
      .catch((err) => {
        console.error('Error adding card:', err);
        alert(err.message || 'Failed to add card.');
      });
  };

  const handleLeaveRetro = () => {
    fetch(`/api/retros/${id}/users`, {
      method: 'DELETE',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username }),
    })
      .then((res) => {
        if (!res.ok) {
          return res.json().then((err) => { throw err; });
        }
        return res.json();
      })
      .then((data) => {
        // Navigate back to retro list
        navigate('/');
      })
      .catch((err) => {
        console.error('Error leaving retro:', err);
        alert(err.message || 'Failed to leave retro.');
      });
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
