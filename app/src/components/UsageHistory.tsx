'use client';

import React, { useState, useEffect } from 'react';
import { getUsageHistory } from '../lib/api';

interface UsageHistoryProps {
  userId: string;
}

const UsageHistory: React.FC<UsageHistoryProps> = ({ userId }) => {
  const [usage, setUsage] = useState<any[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchUsage = async () => {
      try {
        const data = await getUsageHistory(userId);
        setUsage(data);
      } catch (err) {
        setError('Failed to fetch usage history.');
      }
    };
    fetchUsage();
  }, [userId]);

  return (
    <div>
      <h2>Usage History</h2>
      {usage.length > 0 ? (
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr>
              <th style={{ border: '1px solid #ddd', padding: '8px' }}>Timestamp</th>
              <th style={{ border: '1px solid #ddd', padding: '8px' }}>Tokens Used</th>
              <th style={{ border: '1px solid #ddd', padding: '8px' }}>Cost ($)</th>
            </tr>
          </thead>
          <tbody>
            {usage.map((entry, index) => (
              <tr key={index}>
                <td style={{ border: '1px solid #ddd', padding: '8px' }}>{entry.timestamp}</td>
                <td style={{ border: '1px solid #ddd', padding: '8px' }}>{entry.tokens_used}</td>
                <td style={{ border: '1px solid #ddd', padding: '8px' }}>{entry.cost.toFixed(4)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : (
        <p>No usage history available.</p>
      )}
      {error && <p style={{ color: 'red' }}>{error}</p>}
    </div>
  );
};

export default UsageHistory;