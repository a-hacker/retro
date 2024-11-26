// frontend/src/components/UsernameForm.js

import React, { useState } from 'react';
import { TextField, Button, Box } from '@mui/material';
import { useMutation, gql } from '@apollo/client';
import { useSnackbar } from 'notistack';

// Define GraphQL queries and mutations
const CREATE_USER = gql`
  mutation GetRetroById($username: String!) {
    createUser(input: {username: $username}) {
      id
      username
    }
  }
`;

const UsernameForm = ({ onSave }) => {
  const [username, setUsername] = useState('');
  const [savingUser, saveUser] = useState(false);
  const { enqueueSnackbar } = useSnackbar();

  // Mutation to add a user
  const [createUser] = useMutation(CREATE_USER, {
    onCompleted: (data) => {
      console.log(data);
      let newUser = data.createUser;
      sessionStorage.setItem('username', newUser.username);
      sessionStorage.setItem('userid', newUser.id);
      onSave(newUser.username, newUser.id);
      enqueueSnackbar('Created user!', { variant: 'success' });
    },
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to create user.', { variant: 'error' });
    },
  });

  const handleSave = () => {
    const trimmedUsername = username.trim();
    if (trimmedUsername) {
      saveUser(true)
      createUser({variables: {
        username,
      }})
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
