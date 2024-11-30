import React from 'react';
import { Paper, Typography } from '@mui/material';

const CardComponent = ({ text, blurred }) => {
  return (
    <Paper elevation={1} sx={{ p: 1, mb: 1, }}>
      {
        blurred ? 
        <Typography sx={{ filter: 'blur(4px)', userSelect: 'none' }}>{text}</Typography> :
        <Typography>{text}</Typography>
      }
    </Paper>
  );
};

export default CardComponent;
