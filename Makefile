.PHONY: build up down logs restart deploy test clean

# Build Docker image
build:
	docker-compose -f docker-compose.prod.yml build

# Start containers
up:
	docker-compose -f docker-compose.prod.yml up -d

# Stop containers
down:
	docker-compose -f docker-compose.prod.yml down

# View logs
logs:
	docker-compose -f docker-compose.prod.yml logs -f

# Restart containers
restart:
	docker-compose -f docker-compose.prod.yml restart

# Deploy to production
deploy: build up
	@echo "Deployment complete!"
	@echo "API available at: http://localhost:56789"

# Test API
test:
	@echo "Testing health endpoint..."
	@curl -f http://localhost:56789/health || echo "API not responding"
	@echo ""
	@echo "Testing holders endpoint..."
	@curl -f http://localhost:56789/holders/9AvytnUKsLxPxFHFqS6VLxaxt5p6BhYNr53SD2Chpump || echo "API not responding"

# Clean up
clean:
	docker-compose -f docker-compose.prod.yml down -v
	docker rmi solana-holder-bot:latest || true

