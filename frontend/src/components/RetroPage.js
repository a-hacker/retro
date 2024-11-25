// frontend/src/components/RetroPage.js

import React, { useState, useEffect } from 'react';
import {
  Typography,
  Box,
  Grid,
  Button,
} from '@mui/material';
import { useParams, useNavigate } from 'react-router-dom';
import Sidebar from './Sidebar';
import Column from './Column';
import { useQuery, useMutation, gql } from '@apollo/client';

// Define GraphQL queries and mutations
const GET_RETRO_BY_ID = gql`
  query GetRetroById($id: Int!) {
    retroById(id: $id) {
      id
      retroName
      creatorName
      createdAt
      users
      cards {
        good {
          id
          text
        }
        bad {
          id
          text
        }
        needsImprovement {
          id
          text
        }
      }
    }
  }
`;

const ADD_USER = gql`
  mutation AddUser($retroId: Int!, $username: String!) {
    addUser(retroId: $retroId, username: $username)
  }
`;

const LEAVE_RETRO = gql`
  mutation LeaveRetro($retroId: Int!, $username: String!) {
    leaveRetro(input: { retroId: $retroId, username: $username })
  }
`;

const ADD_CARD = gql`
  mutation AddCard($input: AddCardInput!) {
    addCard(input: $input) {
      id
      text
    }
  }
`;

const RetroPage = () => {
  const { id } = useParams();
  const navigate = useNavigate();

  const [newCards, setNewCards] = useState({
    GOOD: '',
    BAD: '',
    NEEDS_IMPROVEMENT: '',
  });

  const username = sessionStorage.getItem('username');

  // Fetch retro details
  const { loading, error, data, refetch } = useQuery(GET_RETRO_BY_ID, {
    variables: { id: parseInt(id, 10) },
    pollInterval: 5000, // Polling for real-time updates
  });

  // Mutation to add a user
  const [addUser] = useMutation(ADD_USER, {
    onCompleted: () => {
      refetch();
    },
  });

  // Mutation to leave retro
  const [leaveRetro] = useMutation(LEAVE_RETRO, {
    onCompleted: () => {
      navigate('/');
    },
  });

  // Mutation to add a card
  const [addCard] = useMutation(ADD_CARD, {
    onCompleted: () => {
      refetch();
    },
  });

  useEffect(() => {
    if (!username) {
      alert('Username not found. Please set your username on the homepage.');
      navigate('/');
      return;
    }

    // Automatically add the user to the retro if not already a participant
    if (data && data.retroById && !data.retroById.users.includes(username)) {
      addUser({
        variables: {
          retroId: data.retroById.id,
          username,
        },
      }).catch((err) => {
        console.error('Error adding user:', err);
        alert(err.message || 'Failed to join retro.');
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data, username]);

  if (loading) return <Typography>Loading retro details...</Typography>;
  if (error) return <Typography>Error loading retro details.</Typography>;

  const retro = data.retroById;

  const handleAddCard = (category, text) => {
    if (!text) {
      alert('Please enter some text for the card.');
      return;
    }

    addCard({
      variables: {
        input: {
          retroId: retro.id,
          category,
          text,
        },
      },
    })
      .then(() => {
        setNewCards((prev) => ({ ...prev, [category]: '' }));
      })
      .catch((err) => {
        console.error('Error adding card:', err);
        alert(err.message || 'Failed to add card.');
      });
  };

  const handleLeaveRetro = () => {
    leaveRetro({
      variables: {
        retroId: retro.id,
        username,
      },
    })
      .catch((err) => {
        console.error('Error leaving retro:', err);
        alert(err.message || 'Failed to leave retro.');
      });
  };

  return (
    <Box display="flex" height="100vh">
      <Sidebar users={retro.users} />

      <Box flexGrow={1} p={4} overflow="auto">
        <Box display="flex" justifyContent="space-between" alignItems="center" mb={4}>
          <Typography variant="h4">{retro.retroName}</Typography>
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
              cards={retro.cards.good}
              newCardText={newCards.good}
              onNewCardTextChange={(text) => setNewCards((prev) => ({ ...prev, good: text }))}
              onAddCard={(text) => handleAddCard('GOOD', text)}
            />
          </Grid>
          <Grid item xs={12} md={4}>
            <Column
              title="Bad"
              cards={retro.cards.bad}
              newCardText={newCards.bad}
              onNewCardTextChange={(text) => setNewCards((prev) => ({ ...prev, bad: text }))}
              onAddCard={(text) => handleAddCard('BAD', text)}
            />
          </Grid>
          <Grid item xs={12} md={4}>
            <Column
              title="Needs Improvement"
              cards={retro.cards.needsImprovement}
              newCardText={newCards.needsImprovement}
              onNewCardTextChange={(text) =>
                setNewCards((prev) => ({ ...prev, needsImprovement: text }))
              }
              onAddCard={(text) => handleAddCard('NEEDS_IMPROVEMENT', text)}
            />
          </Grid>
        </Grid>
      </Box>
    </Box>
  );
};

export default RetroPage;
