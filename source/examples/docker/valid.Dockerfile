# Example of a valid Dockerfile with multi-stage build
# This Dockerfile demonstrates best practices for a Node.js application

# -----------------------------
# Stage 1: Development dependencies
# -----------------------------
FROM node:18-alpine AS deps

# Set working directory
WORKDIR /app

# Copy package files first (for better caching)
COPY package.json package-lock.json ./

# Install dependencies
RUN npm ci

# -----------------------------
# Stage 2: Build the application
# -----------------------------
FROM node:18-alpine AS builder

WORKDIR /app

# Copy dependencies from previous stage
COPY --from=deps /app/node_modules ./node_modules

# Copy source code
COPY . .

# Build the application
RUN npm run build

# -----------------------------
# Stage 3: Production image
# -----------------------------
FROM node:18-alpine AS runner

# Set non-root user for better security
RUN addgroup --system --gid 1001 nodejs \
    && adduser --system --uid 1001 appuser \
    && mkdir -p /app/data \
    && chown -R appuser:nodejs /app

# Set working directory
WORKDIR /app

# Set environment to production
ENV NODE_ENV production

# Set proper permissions
COPY --from=builder --chown=appuser:nodejs /app/package*.json ./
COPY --from=builder --chown=appuser:nodejs /app/node_modules ./node_modules
COPY --from=builder --chown=appuser:nodejs /app/dist ./dist
COPY --from=builder --chown=appuser:nodejs /app/public ./public

# Create volume for persistent data
VOLUME /app/data

# Expose the port the app runs on
EXPOSE 3000

# Switch to non-root user
USER appuser

# Use an init system
ENTRYPOINT ["node", "dist/main.js"]

# Set health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD node -e "try { require('http').get('http://localhost:3000/health', res => res.statusCode === 200 ? process.exit(0) : process.exit(1)); } catch (e) { process.exit(1); }"

# Define labels for better metadata
LABEL org.opencontainers.image.source="https://github.com/yourusername/app" \
      org.opencontainers.image.description="Example Node.js application" \
      org.opencontainers.image.licenses="MIT" \
      maintainer="Synx Team <info@example.com>" \
      version="1.0.0"

