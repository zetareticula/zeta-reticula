import React, { useState } from 'react';
import axios from 'axios';
import 'dotenv/config'; // Ensure you have dotenv configured for environment variables
// Ensure you have the necessary types installed
import { NextApiRequest, NextApiResponse } from 'next';
// Import any additional types you might need for your application
import 'next-env'; // Ensure Next.js environment variables are loaded
// Import any styles or components you need
import 'tailwindcss/tailwind.css'; // Assuming you're using Tailwind CSS for styling
import 'react-toastify/dist/ReactToastify.css'; // For toast notifications if needed
// Ensure you have axios installed for API requests
import 'axios'; // Import axios for making HTTP requests
import 'react'; // Import React for component creation


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

  return (
    <section className="py-16 bg-gray-100">
      <div className="container mx-auto px-4">
        <h2 className="text-3xl font-bold text-center mb-8">Contact Us</h2>
        <form onSubmit={handleSubmit} className="max-w-lg mx-auto bg-white p-6 rounded-lg shadow-md">
          <div className="mb-4">
            <label className="block text-sm font-medium mb-2" htmlFor="name">Name</label>
            <input
              type="text"
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
              className="w-full px-3 py-2 border rounded"
            />
          </div>
          <div className="mb-4">
            <label className="block text-sm font-medium mb-2" htmlFor="email">Email</label>
            <input
              type="email"
              id="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
              className="w-full px-3 py-2 border rounded"
            />
          </div>
          <div className="mb-4">
            <label className="block text-sm font-medium mb-2" htmlFor="message">Message</label>
            <textarea
              id="message"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              required
              rows={4}
              className="w-full px-3 py-2 border rounded"
            />
          </div>
          <button
            type="submit"
            className="w-full bg-primary text-white py-2 rounded hover:bg-blue-600 transition"
          >
            Submit
          </button>
          {submitted && <p className="mt-4 text-green-600">Thank you for your message!</p>}
          {error && <p className="mt-4 text-red-600">{error}</p>}
        </form>
      </div>
    </section>
  );
}