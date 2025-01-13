import React, { useState } from 'react';
import { Paper, Typography, IconButton, TextField } from '@mui/material';
import Add from '@mui/icons-material/Add';
import Edit from '@mui/icons-material/Edit';
import Remove from '@mui/icons-material/Remove';
import Save from '@mui/icons-material/Save';
import { useMutation, gql } from '@apollo/client';
import { useSnackbar } from 'notistack';

const VOTE_CARD = gql`
  mutation VoteCard($retroId: Uuid!, $userId: Uuid!, $cardId: Uuid!, $vote: Boolean!) {
    voteCard(retroId: $retroId, userId: $userId, cardId: $cardId, vote: $vote) {
      user {
        id
        username
      }
      votes {
        id
      }
    }
  }
`;

const EDIT_CARD = gql`
  mutation EditCard($retroId: Uuid!, $cardId: Uuid!, $text: String!) {
    editCard(retroId: $retroId, cardId: $cardId, text: $text) {
      id
      text
      owned
    }
  }
`;

const CreateCardComponent = ({card, user, retroId}) => {
  const { enqueueSnackbar } = useSnackbar();
  const [edittingCard, setEditStatus] = useState(false);
  const [newCardText, setNewCardText] = useState(card.text);

  const [editCard] = useMutation(EDIT_CARD, {
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to edit card.', { variant: 'error' });
    },
  });

  const handleEditCard = (cardId, text) => {
    editCard({
      variables: {
        retroId: retroId,
        cardId: cardId,
        text: text
      },
    }).catch((err) => {
      console.error('Error editting card:', err);
    })
    setEditStatus(false)
  }

  if (!card.owned) {
    return (
      <Paper elevation={1} sx={{ p: 1, mb: 1, }}>
        <Typography sx={{ filter: 'blur(4px)', userSelect: 'none' }}>{card.text}</Typography>
      </Paper>
    )
  };

  if (edittingCard) {
    return (
      <Paper elevation={1} sx={{ p: 1, mb: 1, }}>
        <TextField
          variant="outlined"
          size="small"
          value={newCardText}
          onChange={(e) => setNewCardText(e.target.value)}
          fullWidth
        />
        <IconButton size='small'>
          <Save onClick={() => handleEditCard(card.id, newCardText)}/>
        </IconButton>
      </Paper>
    )
  }

  return (
    <Paper elevation={1} sx={{ p: 1, mb: 1, flex: true }}>
      <Typography>{card.text}</Typography>
      <IconButton size='small'>
        <Edit onClick={() => setEditStatus(true)}/>
      </IconButton>
    </Paper>
  );
};

const VoteCardComponent = ({card, participants, user, retroId}) => {
  // Mutation to vote on a card
  const { enqueueSnackbar } = useSnackbar();
  const [voteCard] = useMutation(VOTE_CARD, {
    onError: (err) => {
      enqueueSnackbar(err.message || 'Failed to vote on card.', { variant: 'error' });
    },
  });

  const handleVoteCard = (cardId, vote) => {
    voteCard({
      variables: {
        retroId: retroId,
        userId: user,
        cardId: cardId,
        vote: vote
      },
    }).catch((err) => {
      console.error('Error voting on card:', err);
    })
  }

  let myParticipant = participants.find(p => p.user.id === user) || {"votes": []}

  return (
    <Paper elevation={1} sx={{ p: 1, mb: 1, }}>
      <Typography>{card.text}</Typography>
      {!myParticipant.votes.some(c => c.id === card.id) ? 
        <IconButton size='small'>
          <Add onClick={() => handleVoteCard(card.id, true)}/>
        </IconButton> :
        <IconButton size='small'>
          <Remove onClick={() => handleVoteCard(card.id, false)}/>
        </IconButton> 
      }
    </Paper>
  );
};

const ReviewCardComponent = ({card, participants}) => {
  let cardVotes = {}
  participants.forEach(participant => {
    participant.votes.forEach(vote => {
      let card_id = vote.id
      if (card_id in cardVotes) {
        cardVotes[card_id] = cardVotes[card_id] + 1
      } else {
        cardVotes[card_id] = 1
      }
    })
  })

  return (
    <Paper elevation={1} sx={{ p: 1, mb: 1, }}>
      <Typography>{card.text}</Typography>
      <Typography>Votes: {cardVotes[card.id] || 0 }</Typography>
    </Paper>
  );
};

const DefaultCardComponent = ({card}) => {
  return (
    <Paper elevation={1} sx={{ p: 1, mb: 1, }}>
      <Typography>{card.text}</Typography>
    </Paper>
  );
};


const CardComponent = ({ card, participants, step, user, retroId }) => {
  switch(step) {
    case "Writing":
      return <CreateCardComponent card={card} user={user} retroId={retroId}/>
    case "Voting":
      return <VoteCardComponent card={card} participants={participants} user={user} retroId={retroId} />
    case "Reviewing":
      return <ReviewCardComponent card={card} participants={participants} />
    default:
      return <DefaultCardComponent card={card} />
  }
};

export default CardComponent;
