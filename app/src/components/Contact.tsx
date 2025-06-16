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
      const response = await axios.post('http://localhost:8080/feedback', {
        name,
        email,
        message,
      }, {
        headers: { 'Authorization': `Bearer ${process.env.NEXT_PUBLIC_API_KEY}` }, // Optional API key
      });
      if (response.data.status === 'success') {
        setSubmitted(true);
        setName('');
        setEmail('');
        setMessage('');
      }
    } catch (err) {
      setError('Failed to submit. Please try again.');
      console.error(err);
    }
  };

  return (
    <section className="bg-secondary text-white py-12">
      <div className="container mx-auto px-4 text-center">
        <h2 className="text-3xl font-bold mb-6">Get in Touch</h2>
        {submitted ? (
          <p className="text-lg">Thank you for your message! We’ll get back to you soon.</p>
        ) : (
          <form onSubmit={handleSubmit} className="max-w-lg mx-auto">
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Your Name"
              className="w-full p-3 mb-4 rounded-lg"
              required
            />
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="Your Email"
              className="w-full p-3 mb-4 rounded-lg"
              required
            />
            <textarea
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              placeholder="Your Message"
              className="w-full p-3 mb-4 rounded-lg"
              rows={4}
              required
            />
            {error && <p className="text-red-300 mb-4">{error}</p>}
            <button
              type="submit"
              className="bg-accent text-white px-6 py-3 rounded-lg hover:bg-yellow-500 transition"
            >
              Send Message
            </button>
          </form>
        )}
      </div>
    </section>
  );
};

export default Contact;