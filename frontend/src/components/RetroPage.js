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
import { useSnackbar } from 'notistack';

// Define GraphQL queries and mutations
const GET_RETRO_BY_ID = gql`
  query GetRetroById($id: Uuid!) {
    retroById(id: $id) {
      id
      retroName
      creator {
        id
        username
      }
      step
      createdAt
      users {
        username
      }
      lanes {
        id
        title
        priority
        cards {
          id
          creator {
            id
          }
          text
        }
      }
    }
  }
`;

const ADD_USER = gql`
  mutation AddUser($retroId: Uuid!, $userId: Uuid!) {
    enterRetro(retroId: $retroId, userId: $userId) {
      username
    }
  }
`;

const LEAVE_RETRO = gql`
  mutation LeaveRetro($retroId: Uuid!, $userId: Uuid!) {
    leaveRetro(retroId: $retroId, userId: $userId) {
      username
    }
  }
`;

const ADD_CARD = gql`
  mutation AddCard($input: AddCardInput!) {
    addCard(input: $input) {
      id
      creator {
        id
      }
      text
    }
  }
`;

const CARD_ADDED_SUBSCRIPTION = gql`
  subscription OnCardAdded($retroId: Uuid!) {
    cardAdded(retroId: $retroId) {
      ... on CardAdded {
        retro {
          id
        }
        lane {
          id
          cards {
            id
            creator {
              id
            }
            text
          }
        }
        card {
          id
          creator {
            id
          }
          text
        }
      }
    }
  }
`;

const USER_LIST_UPDATED_SUBSCRIPTION = gql`
  subscription OnUserListUpdated($retroId: Uuid!) {
    userListUpdated(retroId: $retroId) {
      ... on UserListUpdated {
        users {
          username
        }
      }
    }
  }
`;

const CardBox = ({ retro, username, user_id, subscribeToNewCards, handleLeaveRetro }) => {
  useEffect(() => subscribeToNewCards(), [subscribeToNewCards]);
  const { enqueueSnackbar } = useSnackbar();

  const [newCards, setNewCards] = useState({});

  // Mutation to add a card
  const [addCard] = useMutation(ADD_CARD, {
    onCompleted: () => {
      enqueueSnackbar('Card added successfully!', { variant: 'success' });
    },
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to add card.', { variant: 'error' });
    },
  });

  const handleAddCard = (laneId, text) => {
    if (!text) {
      enqueueSnackbar('Please enter some text for the card.', { variant: 'warning' });
      return;
    }

    addCard({
      variables: {
        input: {
          retroId: retro.id,
          laneId,
          creatorId: user_id,
          text,
        },
      },
    })
      .then(() => {
        setNewCards((prev) => ({ ...prev, [laneId]: '' }));
      })
      .catch((err) => {
        console.error('Error adding card:', err);
      });
  };

  return (
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
        {retro.lanes.map((lane, i) => 
          <Grid item xs={12} md={4}>
            <Column
              title={lane.title}
              cards={lane.cards}
              newCardText={newCards[lane.id]}
              onNewCardTextChange={(text) => setNewCards((prev) => ({ ...prev, [lane.id]: text }))}
              onAddCard={(text) => handleAddCard(lane.id, text)}
            />
          </Grid>
        )}
      </Grid>
    </Box>
  )
}

const RetroPage = () => {
  const { id } = useParams();
  const navigate = useNavigate();
  const { enqueueSnackbar } = useSnackbar();

  const username = sessionStorage.getItem('username');
  const user_id = sessionStorage.getItem('userid');
  const [enteredRetro, enterRetro] = useState(false);

  // Mutation to add a user
  const [addUser] = useMutation(ADD_USER, {
    onCompleted: () => {
      enqueueSnackbar('Joined retro successfully!', { variant: 'success' });
    },
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to join retro.', { variant: 'error' });
    },
  });

  if (!enteredRetro){
    enterRetro(true);
    addUser({
      variables: {
        retroId: id,
        userId: user_id,
      },
    });
  }

  // Fetch retro details
  const { loading, error, data, subscribeToMore } = useQuery(GET_RETRO_BY_ID, {
    variables: { id },
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

  if (loading) return <Typography>Loading retro details...</Typography>;
  if (error) return <Typography>Error loading retro details.</Typography>;
  const retro = data.retroById;

  // Subscribe to user list updates
  const subscribeUsers = () => subscribeToMore({
    document: USER_LIST_UPDATED_SUBSCRIPTION,
    variables: { retroId: retro.id },
    updateQuery: (prev, { subscriptionData }) => {
      if (!subscriptionData.data) return prev;

      let newRetro = Object.assign({}, prev, {
        retroById: Object.assign({}, prev.retroById, {
          users: subscriptionData.data.userListUpdated.users
        })
      })
      return newRetro
    },
  });

  // Subscribe to new cards
  const subscribeToCards = () => subscribeToMore({
    document: CARD_ADDED_SUBSCRIPTION,
    variables: { retroId: retro.id },
    updateQuery: (prev, { subscriptionData }) => {
      if (!subscriptionData.data || !subscriptionData.data.cardAdded) return prev;

      let cardLane = subscriptionData.data.cardAdded.lane;
      
      let currentLanes = prev.retroById.lanes;
      let newLanes = currentLanes.map((l) => l.id === cardLane.id ? cardLane : l)

      let newRetro = Object.assign({}, prev, {
        retroById: Object.assign({}, prev.retroById, {
          lanes: newLanes
        })
      })

      return newRetro
    },
  });

  const handleLeaveRetro = () => {
    leaveRetro({
      variables: {
        retroId: retro.id,
        userId: user_id,
      },
    })
      .catch((err) => {
        console.error('Error leaving retro:', err);
        alert(err.message || 'Failed to leave retro.');
      });
  };

  return (
    <Box display="flex" height="100vh">
      <Sidebar users={retro.users} subscribeToUsers={subscribeUsers}/>
      <CardBox retro={retro} username={username} user_id={user_id} handleLeaveRetro={handleLeaveRetro} subscribeToNewCards={subscribeToCards} />
    </Box>
  );
};

export default RetroPage;
