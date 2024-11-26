// frontend/src/components/RetroList.js

import React, { useState } from 'react';
import {
  List,
  ListItem,
  ListItemText,
  Button,
  TextField,
  Box,
  Typography,
  Paper,
} from '@mui/material';
import { Link } from 'react-router-dom';
import { useQuery, useMutation, gql } from '@apollo/client';

// Define GraphQL queries and mutations
const GET_ALL_RETROS = gql`
  query GetAllRetros {
    allRetros {
      id
      retroName
      creatorId
      createdAt
      users
    }
  }
`;

const CREATE_RETRO = gql`
  mutation CreateRetro($input: CreateRetroInput!) {
    createRetro(input: $input) {
      id
      retroName
      creatorId
      createdAt
      users
    }
  }
`;

const RetroList = ({ username, user_id }) => {
  const [newRetroName, setNewRetroName] = useState('');

  // Fetch all retros
  const { loading, error, data } = useQuery(GET_ALL_RETROS, {
    pollInterval: 5000, // Polling interval for real-time updates
  });

  // Mutation to create a new retro
  const [createRetro] = useMutation(CREATE_RETRO, {
    // Update the cache to include the new retro
    update(cache, { data: { createRetro } }) {
      const { allRetros } = cache.readQuery({ query: GET_ALL_RETROS });
      cache.writeQuery({
        query: GET_ALL_RETROS,
        data: { allRetros: [...allRetros, createRetro] },
      });
    },
  });

  if (loading) return <Typography>Loading retros...</Typography>;
  if (error) return <Typography>Error fetching retros.</Typography>;

  const handleCreateRetro = () => {
    const trimmedName = newRetroName.trim();
    if (!trimmedName) {
      alert('Please enter a retro name.');
      return;
    }

    createRetro({
      variables: {
        input: {
          retroName: trimmedName,
          creatorId: user_id,
        },
      },
    })
      .then(() => {
        setNewRetroName('');
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
      <Paper elevation={3} sx={{ maxHeight: 400, overflow: 'auto' }}>
        <List>
          {data.allRetros.length === 0 ? (
            <Typography p={2}>No retrospectives available. Create one!</Typography>
          ) : (
            data.allRetros.map((retro) => (
              <ListItem
                key={retro.id}
                button
                component={Link}
                to={`/retros/${retro.id}`}
              >
                <ListItemText
                  primary={retro.retroName}
                  secondary={`Created by: ${retro.creatorId}`}
                />
              </ListItem>
            ))
          )}
        </List>
      </Paper>

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
            onKeyPress={(e) => {
              if (e.key === 'Enter') handleCreateRetro();
            }}
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
