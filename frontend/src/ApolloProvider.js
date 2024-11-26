// frontend/src/ApolloProvider.js

import React from 'react';
import {
  ApolloClient,
  InMemoryCache,
  ApolloProvider as Provider,
  createHttpLink,
  split,
} from '@apollo/client';
import { getMainDefinition } from '@apollo/client/utilities';
import { createClient } from 'graphql-ws';
import { GraphQLWsLink } from '@apollo/client/link/subscriptions';

// Create an http link to the GraphQL server
const httpLink = createHttpLink({
  uri: 'http://localhost:8000/graphql', // Adjust if your backend is hosted elsewhere
});

// WebSocket link for subscriptions
const wsLink = new GraphQLWsLink(
  createClient({
    url: 'ws://localhost:8000/subscriptions', // Backend WebSocket endpoint
  })
);

// Using the ability to split links, send data to each link 
// depending on what kind of operation is being sent.
const splitLink = split(
  ({ query }) => {
    const definition = getMainDefinition(query);
    return (
      definition.kind === 'OperationDefinition' &&
      definition.operation === 'subscription'
    );
  },
  wsLink,
  httpLink,
);

// Initialize Apollo Client
const client = new ApolloClient({
  link: splitLink,
  cache: new InMemoryCache(),
});

// Create ApolloProvider component
const ApolloProvider = ({ children }) => {
  return <Provider client={client}>{children}</Provider>;
};

export default ApolloProvider;
