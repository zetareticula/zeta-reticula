import { NextRequest, NextResponse } from 'next/server';
import { getUsage } from '../../../lib/api';

export async function GET(req: NextRequest, { params }: { params: { userId: string } }) {
  const { userId } = params;
  try {
    const usage = await getUsageHistory(userId);
    return NextResponse.json(usage);
  } catch (error) {
    return NextResponse.json({ error: 'Failed to fetch usage history' }, { status: 500 });
  }
}