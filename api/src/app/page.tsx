import Hero from '@/components/Hero';
import About from '@/components/About';
import Solutions from '@/components/Solutions';
import Contact from '@/components/Contact';
import Subscribe from '@/components/Subscribe';
import Footer from '@/components/Footer';

export default function Home() {
  return (
    <div>
      <Hero />
      <About />
      <Solutions />
      <Contact />
      <Subscribe />
      <Footer />
    </div>
  );
}