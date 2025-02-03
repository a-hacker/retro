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
import StepBar from './StepBar';
import { useQuery, useMutation, gql } from '@apollo/client';
import { useSnackbar } from 'notistack';

// Define GraphQL queries and mutations
const GET_RETRO_BY_ID = gql`
  query GetRetroById($id: String!) {
    retroById(id: $id) {
      id
      retroName
      creator {
        id
        username
        activeUser
      }
      step
      createdAt
      participants {
        user {
          id
          username
          activeUser
        }
      }
      lanes {
        id
        title
        priority
        cards {
          id
          text
          owned
          voted
          votes
        }
      }
    }
  }
`;

const ADD_USER = gql`
  mutation AddUser($retroId: String!) {
    enterRetro(retroId: $retroId) {
      user {
        username
      }
    }
  }
`;

const LEAVE_RETRO = gql`
  mutation LeaveRetro($retroId: String!) {
    leaveRetro(retroId: $retroId) {
      user {
        username
      }
    }
  }
`;

const ADD_CARD = gql`
  mutation AddCard($input: AddCardInput!) {
    addCard(input: $input) {
      id
      text
      owned
      voted
      votes
    }
  }
`;

const UPDATE_STEP = gql`
  mutation UpdateStep($retroId: String!, $step: RetroStep!) {
    updateRetroStep(retroId: $retroId, step: $step) {
      step
    }
  }
`;

const CARD_ADDED_SUBSCRIPTION = gql`
  subscription OnCardAdded($retroId: String!) {
    cardAdded(retroId: $retroId) {
      ... on CardAdded {
        retro {
          id
        }
        lane {
          id
          title
          priority
          cards {
            id
            text
            owned
            voted
            votes
          }
        }
        card {
          id
          text
          owned
          voted
          votes
        }
      }
    }
  }
`;

const USER_LIST_UPDATED_SUBSCRIPTION = gql`
  subscription OnUserListUpdated($retroId: String!) {
    userListUpdated(retroId: $retroId) {
      ... on UserListUpdated {
        participants {
          user {
            id
            username
            activeUser
          }
        }
      }
    }
  }
`;

const UPDATE_STEP_SUBSCRIPTION = gql`
  subscription OnStepUpdated($retroId: String!) {
    stepUpdate(retroId: $retroId) {
      ... on StepUpdated {
        step
      }
    }
  }
`;

const CardBox = ({ retro, username, subscribeToNewCards, subscribeToUsers, handleLeaveRetro }) => {
  useEffect(() => subscribeToNewCards(), [subscribeToNewCards]);
  useEffect(() => subscribeToUsers(), [subscribeToUsers]);
  const { enqueueSnackbar } = useSnackbar();

  const [newCards, setNewCards] = useState({});

  // Mutation to add a card
  const [addCard] = useMutation(ADD_CARD, {
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
              participants={retro.participants}
              newCardText={newCards[lane.id]}
              onNewCardTextChange={(text) => setNewCards((prev) => ({ ...prev, [lane.id]: text }))}
              onAddCard={(text) => handleAddCard(lane.id, text)}
              step={retro.step}
              retroId={retro.id}
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
  const [enteredRetro, enterRetro] = useState(false);

  // Mutation to add a user
  const [addUser] = useMutation(ADD_USER, {
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to join retro.', { variant: 'error' });
    },
  });

  if (!enteredRetro){
    enterRetro(true);
    addUser({
      variables: {
        retroId: id,
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
      navigate('/retro');
    },
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to leave retro.', { variant: 'error' });
    },
  });

  // Mutation to update retro step
  const [updateRetroStep] = useMutation(UPDATE_STEP, {
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to change retro step.', { variant: 'error' });
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
          participants: subscriptionData.data.userListUpdated.participants
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

  const subscribeToStep = () => subscribeToMore({
    document: UPDATE_STEP_SUBSCRIPTION,
    variables: { retroId: retro.id },
    updateQuery: (prev, { subscriptionData }) => {
      if (!subscriptionData.data || !subscriptionData.data.stepUpdate) return prev;

      let newStep = subscriptionData.data.stepUpdate.step;
      let newRetro = Object.assign({}, prev, {
        retroById: Object.assign({}, prev.retroById, {
          step: newStep
        })
      })

      return newRetro
    },
  });

  const handleLeaveRetro = () => {
    leaveRetro({
      variables: {
        retroId: retro.id,
      },
    })
      .catch((err) => {
        console.error('Error leaving retro:', err);
        alert(err.message || 'Failed to leave retro.');
      });
  };

  const handleRetroStepClick = (newStep) => {
    updateRetroStep({
      variables: {
        retroId: retro.id,
        step: newStep,
      },
    }).catch((err) => {
      console.error('Error updating retro step:', err);
    });
  }

  return (
    <Box display="flex" height="100vh">
      <Sidebar participants={retro.participants} subscribeToUsers={subscribeUsers} />
      <CardBox retro={retro} username={username} handleLeaveRetro={handleLeaveRetro} subscribeToUsers={subscribeUsers} subscribeToNewCards={subscribeToCards} />
      <StepBar currentStep={retro.step} handleRetroStepClick={handleRetroStepClick} subscribeToStep={subscribeToStep} />
    </Box>
  );
};

export default RetroPage;
