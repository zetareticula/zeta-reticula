import { NextResponse } from 'next/server';

// Mock usage history
const mockUsageHistory = [
  {
    id: 'inf_123',
    timestamp: new Date(Date.now() - 3600000).toISOString(),
    model: '1',
    prompt: 'Hello, world!',
    tokens_used: 15
  },
  {
    id: 'inf_124',
    timestamp: new Date(Date.now() - 7200000).toISOString(),
    model: '2',
    prompt: 'What is the meaning of life?',
    tokens_used: 42
  }
];

export async function GET() {
  try {
    return NextResponse.json(mockUsageHistory, { status: 200 });
  } catch (error) {
    console.error('Error fetching usage history:', error);
    return NextResponse.json(
      { error: 'Failed to fetch usage history' },
      { status: 500 }
    );
  }
}
