// frontend/src/ApolloProvider.js

import React from 'react';
import {
  ApolloClient,
  InMemoryCache,
  ApolloProvider as Provider,
  createHttpLink,
} from '@apollo/client';

// Create an http link to the GraphQL server
const httpLink = createHttpLink({
  uri: 'http://localhost:8000/graphql', // Adjust if your backend is hosted elsewhere
});

// Initialize Apollo Client
const client = new ApolloClient({
  link: httpLink,
  cache: new InMemoryCache(),
});

// Create ApolloProvider component
const ApolloProvider = ({ children }) => {
  return <Provider client={client}>{children}</Provider>;
};

export default ApolloProvider;
