import React, { useState, useEffect } from 'react';
import { loadStripe } from '@stripe/stripe-js';
import axios from 'axios';

const stripePromise = loadStripe(process.env.NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY!);

const Subscribe: React.FC = () => {
  const [email, setEmail] = useState('');
  const [plan, setPlan] = useState('pro');
  const [submitted, setSubmitted] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSubmitted(false);

    try {
      const stripe = await stripePromise;
      if (!stripe) throw new Error('Stripe failed to load');

      const response = await axios.post(`${process.env.NEXT_PUBLIC_API_URL}/subscribe`, {
        email,
        plan,
      }, {
        headers: { 'Authorization': `Bearer ${process.env.NEXT_PUBLIC_API_KEY}` },
      });

      if (response.data.status === 'success' && response.data.checkout_url) {
        await stripe.redirectToCheckout({ sessionId: response.data.subscription_id });
      }
    } catch (err) {
      setError('Subscription failed. Please try again.');
      console.error(err);
    }
  };

  return (
    <section className="bg-primary text-white py-12">
      <div className="container mx-auto px-4 text-center">
        <h2 className="text-3xl font-bold mb-6">Join Zeta Reticula</h2>
        {submitted ? (
          <p className="text-lg">Subscribed successfully! Check your email for details.</p>
        ) : (
          <form onSubmit={handleSubmit} className="max-w-lg mx-auto">
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="Your Email"
              className="w-full p-3 mb-4 rounded-lg"
              required
            />
            <select
              value={plan}
              onChange={(e) => setPlan(e.target.value)}
              className="w-full p-3 mb-4 rounded-lg"
            >
              <option value="basic">Basic (Free)</option>
              <option value="pro">Pro ($29/month)</option>
              <option value="enterprise">Enterprise (Custom)</option>
            </select>
            {error && <p className="text-red-300 mb-4">{error}</p>}
            <button
              type="submit"
              className="bg-accent text-white px-6 py-3 rounded-lg hover:bg-yellow-500 transition"
            >
              Subscribe Now
            </button>
          </form>
        )}
      </div>
    </section>
  );
};

export default Subscribe;