import { NextResponse } from 'next/server';
import jwt from 'jsonwebtoken';

// Mock user data (in production, use a proper database)
const MOCK_USER = {
  id: '1',
  username: 'testuser',
  email: 'test@example.com',
  // In a real app, never store passwords in plain text!
  password: 'testpass123'
};

export async function POST(request: Request) {
  try {
    const { username, password } = await request.json();
    
    // In a real app, verify credentials against a database
    if (username !== MOCK_USER.username || password !== MOCK_USER.password) {
      return NextResponse.json(
        { error: 'Invalid credentials' },
        { status: 401 }
      );
    }
    
    // Create a JWT token
    const token = jwt.sign(
      { userId: MOCK_USER.id, username: MOCK_USER.username },
      process.env.JWT_SECRET || 'dev-secret-key',
      { expiresIn: '1h' }
    );
    
    return NextResponse.json({
      token,
      user: {
        id: MOCK_USER.id,
        username: MOCK_USER.username,
        email: MOCK_USER.email
      }
    }, { status: 200 });
    
  } catch (error) {
    console.error('Login error:', error);
    return NextResponse.json(
      { error: 'Login failed' },
      { status: 500 }
    );
  }
}
