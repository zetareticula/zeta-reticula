import { NextResponse } from 'next/server';

// Mock inference function
async function runInference(prompt: string, modelId: string = '1') {
  // In a real implementation, this would call the actual model
  return {
    id: `inf_${Date.now()}`,
    model: modelId,
    prompt: prompt,
    response: `This is a mock response to: ${prompt}`,
    tokens_used: Math.floor(Math.random() * 100) + 10,
    timestamp: new Date().toISOString()
  };
}

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

// Handle POST /api/inference
export async function POST(request: Request) {
  try {
    const { prompt, modelId } = await request.json();
    
    if (!prompt) {
      return NextResponse.json(
        { error: 'Prompt is required' },
        { status: 400 }
      );
    }
    
    const result = await runInference(prompt, modelId);
    
    return NextResponse.json(result, { status: 200 });
  } catch (error) {
    console.error('Error running inference:', error);
    return NextResponse.json(
      { error: 'Failed to process inference request' },
      { status: 500 }
    );
  }
}

// Handle GET /api/inference/usage
export async function GET(request: Request) {
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
