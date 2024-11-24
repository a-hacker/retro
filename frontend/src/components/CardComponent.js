// frontend/src/components/CardComponent.js

import React from 'react';
import { Paper, Typography } from '@mui/material';

const CardComponent = ({ text }) => {
  return (
    <Paper elevation={1} sx={{ p: 1, mb: 1 }}>
      <Typography>{text}</Typography>
    </Paper>
  );
};

export default CardComponent;
