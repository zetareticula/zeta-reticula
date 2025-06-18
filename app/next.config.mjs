/** @type {import('next').NextConfig} */
const nextConfig = {
    reactStrictMode: true,
    experimental: {
        appDir: true,
        serverComponentsExternalPackages: ['@prisma/client'],
    },
    transpilePackages: ['@prisma/client'],
    images: {
        remotePatterns: [
            {
                protocol: 'https',
                hostname: 'images.unsplash.com',
                port: '',
                pathname: '/**',
            },
            {
                protocol: 'https',
                hostname: 'cdn.discordapp.com',
                port: '',
                pathname: '/**',
            },
        ],
    },
    output: 'standalone',
  };

// This is a Next.js configuration file that sets up various options for the Next.js application.
// It enables React strict mode, experimental features, transpiles the Prisma client package,
// configures image loading from specific remote patterns, and sets the output to 'standalone'.
  
// The `reactStrictMode` option helps identify potential problems in the application by activating additional checks and warnings.
// The `experimental` section allows the use of experimental features, such as server components.
// The `transpilePackages` option ensures that the Prisma client is transpiled correctly for use in the application.

  export default nextConfig;