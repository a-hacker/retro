// frontend/src/components/Column.js

import React from 'react';
import { Paper, Typography, List, TextField, Button, Box } from '@mui/material';
import CardComponent from './CardComponent';

const Column = ({ title, cards, newCardText, onNewCardTextChange, onAddCard }) => {
  return (
    <Paper elevation={3} sx={{ p: 2, height: '100%' }}>
      <Typography variant="h6" align="center" gutterBottom>
        {title}
      </Typography>
      <List sx={{ minHeight: 200, maxHeight: 400, overflow: 'auto' }}>
        {cards.map((card, index) => (
          <CardComponent key={index} text={card} />
        ))}
      </List>
      <Box display="flex" mt={2}>
        <TextField
          variant="outlined"
          size="small"
          placeholder={`Add a card to ${title}...`}
          value={newCardText}
          onChange={(e) => onNewCardTextChange(e.target.value)}
          fullWidth
        />
        <Button variant="contained" color="primary" onClick={onAddCard} sx={{ ml: 1 }}>
          Add
        </Button>
      </Box>
    </Paper>
  );
};

export default Column;
