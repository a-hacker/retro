import React from 'react';
import { Paper, Typography, IconButton } from '@mui/material';
import Add from '@mui/icons-material/Add';
import { useMutation, gql } from '@apollo/client';
import { useSnackbar } from 'notistack';

const VOTE_CARD = gql`
  mutation VoteCard($retroId: Uuid!, $userId: Uuid!, $cardId: Uuid!) {
    voteCard(retroId: $retroId, userId: $userId, cardId: $cardId) {
      id
      creator {
        id
      }
      text
      votes
    }
  }
`;


const CardComponent = ({ card, step, user, retroId }) => {
  const { enqueueSnackbar } = useSnackbar();

  // Mutation to vote on a card
  const [voteCard] = useMutation(VOTE_CARD, {
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to vote on card.', { variant: 'error' });
    },
  });

  const handleVoteCard = (cardId) => {
    voteCard({
      variables: {
        retroId: retroId,
        userId: user,
        cardId: cardId
      },
    }).catch((err) => {
      console.error('Error voting on card:', err);
    })
  }

  return (
    <Paper elevation={1} sx={{ p: 1, mb: 1, }}>
      {
        step === "Writing" && card.creator.id !== user ? 
        <Typography sx={{ filter: 'blur(4px)', userSelect: 'none' }}>{card.text}</Typography> :
        <Typography>{card.text}</Typography>
      }
      {
        step === "Voting" ?
        <IconButton size='small'>
          <Add onClick={() => handleVoteCard(card.id)}/>
          </IconButton> : null
      }
      {
        step === "Reviewing" ?
        <Typography>Votes: {card.votes}</Typography> : null
      }
    </Paper>
  );
};

export default CardComponent;
