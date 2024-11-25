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
import { useQuery, useMutation, useSubscription, gql } from '@apollo/client';
import { useSnackbar } from 'notistack';

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

const CARD_ADDED_SUBSCRIPTION = gql`
  subscription OnCardAdded($retroId: Int!) {
    cardAdded(retroId: $retroId) {
      retroId
      category
      card {
        id
        text
      }
    }
  }
`;

const USER_LIST_UPDATED_SUBSCRIPTION = gql`
  subscription OnUserListUpdated($retroId: Int!) {
    userListUpdated(retroId: $retroId) {
      users
    }
  }
`;

const RetroPage = () => {
  const { id } = useParams();
  const navigate = useNavigate();
  const { enqueueSnackbar } = useSnackbar();

  const [newCards, setNewCards] = useState({
    GOOD: '',
    BAD: '',
    NEEDS_IMPROVEMENT: '',
  });

  const username = sessionStorage.getItem('username');

  // Fetch retro details
  const { loading, error, data, subscribeToMore } = useQuery(GET_RETRO_BY_ID, {
    variables: { id: parseInt(id, 10) }
  });

  // Mutation to add a user
  const [addUser] = useMutation(ADD_USER, {
    onCompleted: () => {
      enqueueSnackbar('Joined retro successfully!', { variant: 'success' });
    },
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to join retro.', { variant: 'error' });
    },
  });

  // Mutation to leave retro
  const [leaveRetro] = useMutation(LEAVE_RETRO, {
    onCompleted: () => {
      enqueueSnackbar('Left retro successfully!', { variant: 'success' });
      navigate('/');
    },
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to leave retro.', { variant: 'error' });
    },
  });

  // Mutation to add a card
  const [addCard] = useMutation(ADD_CARD, {
    onCompleted: () => {
      enqueueSnackbar('Card added successfully!', { variant: 'success' });
    },
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to add card.', { variant: 'error' });
    },
  });

  // Subscription for card additions
  useEffect(() => {
    if (loading || error || !data) return;

    const retroId = data.retroById.id;
    
    // Subscribe to card additions
    const unsubscribeCard = subscribeToMore({
      document: CARD_ADDED_SUBSCRIPTION,
      variables: { retroId: retroId },
      updateQuery: (prev, { subscriptionData }) => {
        if (!subscriptionData.data) return prev;
        const newUpdate = subscriptionData.data.cardAdded;

        const updatedCards = { ...prev.retroById.cards };

        switch (newUpdate.category) {
          case 'good':
            updatedCards.good.push(newUpdate.card);
            break;
          case 'bad':
            updatedCards.bad.push(newUpdate.card);
            break;
          case 'needs_improvement':
            updatedCards.needs_improvement.push(newUpdate.card);
            break;
          default:
            break;
        }

        return {
          ...prev,
          retroById: {
            ...prev.retroById,
            cards: updatedCards,
          },
        };
      },
    });

    // Subscribe to user list updates
    const unsubscribeUser = subscribeToMore({
      document: USER_LIST_UPDATED_SUBSCRIPTION,
      variables: { retroId: retroId },
      updateQuery: (prev, { subscriptionData }) => {
        if (!subscriptionData.data) return prev;
        const newUpdate = subscriptionData.data.userListUpdated;

        return {
          ...prev,
          retroById: {
            ...prev.retroById,
            users: newUpdate.users,
          },
        };
      },
    });

    // Clean up subscriptions on unmount
    return () => {
      unsubscribeCard();
      unsubscribeUser();
    };
  }, [error, data, loading, subscribeToMore]);

  useEffect(() => {
    if (loading || error || !data) return;

    const retro = data.retroById;

    // Automatically add the user to the retro if not already a participant
    if (!retro.users.includes(username)) {
      addUser({
        variables: {
          retroId: retro.id,
          username,
        },
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loading, error, data, addUser, username]);

  if (loading) return <Typography>Loading retro details...</Typography>;
  if (error) return <Typography>Error loading retro details.</Typography>;
  const retro = data.retroById;

  const handleAddCard = (category, text) => {
    if (!text) {
      enqueueSnackbar('Please enter some text for the card.', { variant: 'warning' });
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

  // Subscribe to user list updates
  const subscribeUsers = () => subscribeToMore({
    document: USER_LIST_UPDATED_SUBSCRIPTION,
    variables: { retroId: retro.id },
    updateQuery: (prev, { subscriptionData }) => {
      if (!subscriptionData.data) return prev;
      return subscriptionData.data.userListUpdated.users;
    },
  });

  return (
    <Box display="flex" height="100vh">
      <Sidebar users={retro.users} subscribeToUsers={subscribeUsers} />

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
