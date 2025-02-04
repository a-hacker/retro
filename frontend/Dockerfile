# Build Stage
FROM node:22-alpine AS build
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
RUN npm run build
 
# Production Stage
FROM nginx:stable-alpine AS production
COPY --from=build /app/build /usr/share/nginx/html/retro
COPY nginx.conf /etc/nginx/conf.d/default.conf

COPY ./populate_env.sh /usr/share/nginx/html/retro/populate_env.sh
RUN chmod +x /usr/share/nginx/html/retro/populate_env.sh

# Add bash
RUN apk add --no-cache bash

EXPOSE 80

WORKDIR /usr/share/nginx/html/retro/
CMD ["/bin/bash", "-c", "/usr/share/nginx/html/retro/populate_env.sh && nginx -g \"daemon off;\""]