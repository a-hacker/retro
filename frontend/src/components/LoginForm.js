import React, { useState } from 'react';
import { TextField, Button, Box, Typography } from '@mui/material';
import axios from 'axios';

const LoginForm = ({ onSave }) => {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState(null);

  const backend_address = window._env_.REACT_APP_AUTH_URI || 'http://localhost:8000';

  const handleLogin = async (e) => {
    e.preventDefault();
    try {
      const response = await axios.post(`${backend_address}/api/v1/auth/login`, { username, password});
      
      sessionStorage.setItem('access_token', response.data['token']);
      sessionStorage.setItem('refresh_token', response.data['refresh_token']);
      onSave(username);
    } catch (err) {
      setError('Login failed. Please check your username and password.');
    }
  };

  return (
    <React.Fragment>
    <Box
      component="form"
      onSubmit={handleLogin}
      sx={{ display: 'flex', flexDirection: 'column', gap: 2, maxWidth: 400, margin: 'auto', mt: 4 }}
    >
      <Typography variant="h4" component="h1" gutterBottom>
        Login
      </Typography>

      <TextField
        label="Username"
        value={username}
        onChange={(e) => setUsername(e.target.value)}
        required
      />

      <TextField
        label="Password"
        type="password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        required
      />

      {error && (
        <Typography color="error">
          {error}
        </Typography>
      )}

      <Button variant="contained" color="primary" type="submit">
        Login
      </Button>
    </Box>
    </React.Fragment>
  );
};

export default LoginForm;