// frontend/src/ApolloProvider.js

import React from 'react';
import {
  from,
  ApolloClient,
  ApolloLink,
  InMemoryCache,
  ApolloProvider as Provider,
  createHttpLink,
  split,
} from '@apollo/client';
import { getMainDefinition } from '@apollo/client/utilities';
import { createClient } from 'graphql-ws';
import { GraphQLWsLink } from '@apollo/client/link/subscriptions';

const backend_address = process.env.REACT_APP_BACKEND_URI || 'http://localhost:8080';
const websocket_address = process.env.REACT_APP_WEBSOCKET_URI || 'ws://localhost:8080';

// Create an http link to the GraphQL server
const httpLink = createHttpLink({
  uri: `${backend_address}/graphql`,
});

// WebSocket link for subscriptions
const wsLink = new GraphQLWsLink(
  createClient({
    url: `${websocket_address}/subscriptions`,
    connectionParams: () => ({
      access_token: sessionStorage.getItem('access_token'),
    })
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

const authLink = new ApolloLink((operation, forward) => {
  const access_token = sessionStorage.getItem('access_token')
  const refresh_token = sessionStorage.getItem('refresh_token')

  operation.setContext(({ headers }) => ({ headers: {
    access_token: access_token,
    refresh_token: refresh_token,
    ...headers,
  }}));

  return forward(operation)
})

// Initialize Apollo Client
const client = new ApolloClient({
  link: from([authLink, splitLink]),
  cache: new InMemoryCache(),
});

// Create ApolloProvider component
const ApolloProvider = ({ children }) => {
  return <Provider client={client}>{children}</Provider>;
};

export default ApolloProvider;
