// frontend/src/components/Sidebar.js

import React, { useEffect } from 'react';
import { Box, Typography, List, ListItem, ListItemText } from '@mui/material';

const Sidebar = ({ participants, subscribeToUsers }) => {

  useEffect(() => subscribeToUsers(), []);

  return (
    <Box
      width={{ xs: '100%', md: 250 }}
      bgcolor="#5c6bc0"
      color="#fff"
      p={2}
      sx={{ height: '100vh', position: 'sticky', top: 0 }}
    >
      <Typography variant="h6" gutterBottom>
        Participants
      </Typography>
      <List>
        {participants.length === 0 ? (
          <Typography>No participants yet.</Typography>
        ) : (
          participants.map((user, index) => (
            <ListItem key={index} disablePadding>
              <ListItemText primary={user.user.username} />
            </ListItem>
          ))
        )}
      </List>
    </Box>
  );
};

export default Sidebar;
