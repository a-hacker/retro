docker_build_backend:
	cd backend; docker build -t retro/backend:latest -f Dockerfile .

docker_build_frontend:
	cd frontend; docker build -t retro/frontend:latest -f Dockerfile .

docker_build: docker_build_backend docker_build_frontend

docker_push_backend:
	docker tag retro/backend:latest registry.digitalocean.com/ahacker-images/retro:backend-latest
	docker push registry.digitalocean.com/ahacker-images/retro:backend-latest

docker_push_frontend:
	docker tag retro/frontend:latest registry.digitalocean.com/ahacker-images/retro:frontend-latest
	docker push registry.digitalocean.com/ahacker-images/retro:frontend-latest

docker_push: docker_push_backend docker_push_frontend
