import Link from 'next/link';

export default function Home() {
  return (
    <div className="container">
      <h1>Zeta Reticula Inference Platform</h1>
      <nav>
        <ul>
          <li><Link href="/upload">Upload Model</Link></li>
          <li><Link href="/inference">Run Inference</Link></li>
        </ul>
      </nav>
    </div>
  );
}