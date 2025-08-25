# Zeta Reticula API

> High-performance API for Zeta Reticula's LLM inference and model management

## 📡 API Endpoints

### Health Check
- `GET /api/health` - Check API status

### Authentication
- `POST /api/auth/login` - Authenticate and get JWT token

### Models
- `GET /api/models` - List all available models
- `GET /api/models/:id` - Get details for a specific model
- `POST /api/models` - Upload a new model (requires authentication)

### Inference
- `POST /api/inference` - Run inference with a prompt
- `GET /api/inference/usage` - Get inference usage history

## 🚀 Getting Started

### Prerequisites
- Node.js 18+
- npm or yarn
- PostgreSQL (for production)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/zetareticula/zeta-reticula.git
   cd zeta-reticula/api
   ```

2. Install dependencies:
   ```bash
   npm install
   # or
   yarn install
   ```

3. Set up environment variables:
   ```bash
   cp .env.example .env.local
   # Edit .env.local with your configuration
   ```

4. Start the development server:
   ```bash
   npm run dev
   # or
   yarn dev
   ```

## 🔒 Authentication

### Obtaining a Token
```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "testuser", "password": "testpass123"}'
```

### Using the Token
Include the token in the `Authorization` header:
```
Authorization: Bearer YOUR_JWT_TOKEN
```

## 📚 API Reference

### Health Check
```http
GET /api/health
```
**Response:**
```json
{
  "status": "ok",
  "timestamp": "2025-08-25T02:35:53.981Z"
}
```

### List Models
```http
GET /api/models
```
**Response:**
```json
[
  {
    "id": "1",
    "name": "zeta-reticula-7b",
    "status": "ready"
  },
  {
    "id": "2",
    "name": "zeta-reticula-13b",
    "status": "ready"
  }
]
```

### Run Inference
```http
POST /api/inference
Content-Type: application/json

{
  "prompt": "What is the capital of France?",
  "modelId": "1"
}
```

**Response:**
```json
{
  "id": "inf_1756089355778",
  "model": "1",
  "prompt": "What is the capital of France?",
  "response": "This is a mock response to: What is the capital of France?",
  "tokens_used": 65,
  "timestamp": "2025-08-25T02:35:55.778Z"
}
```

## 🛠️ Development

### Running Tests
```bash
npm test
# or
yarn test
```

### Building for Production
```bash
npm run build
# or
yarn build
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Port to run the server | `3000` |
| `NODE_ENV` | Environment (development/production) | `development` |
| `JWT_SECRET` | Secret for JWT token signing | `dev-secret-key` |
| `DATABASE_URL` | PostgreSQL connection string | - |

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with ❤️ by the Zeta Reticula team
- Powered by Next.js and TypeScript
