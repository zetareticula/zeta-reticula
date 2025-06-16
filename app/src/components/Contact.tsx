import React, { useState } from 'react';
import axios from 'axios';

const Contact: React.FC = () => {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [message, setMessage] = useState('');
  const [submitted, setSubmitted] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSubmitted(false);

    try {
      const response = await axios.post(`${process.env.NEXT_PUBLIC_API_URL}/feedback`, {
        name,
        email,
        message,
      }, {
        headers: { 'Authorization': `Bearer ${process.env.NEXT_PUBLIC_API_KEY}` },
      });
      if (response.data.status === 'success') {
        setSubmitted(true);
        // Optional: Trigger inference request
        await axios.post(`${process.env.NEXT_PUBLIC_API_URL}/infer`, {
          input: message,
          model_name: 'ZetaModel',
          precision: 'high',
        });
      }
    } catch (err) {
      setError('Failed to submit. Please try again.');
      console.error(err);
    }
  };

  // Rest of the component remains unchanged
  // ...
};

export default Contact;