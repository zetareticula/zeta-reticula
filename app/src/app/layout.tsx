import type { Metadata } from 'next'
import './globals.css'
import { Inter } from 'next/font/google'
import React from 'react'
import Footer from '@/components/Footer'
import Hero from '@/components/Hero'
import About from '@/components/About'
import Solutions from '@/components/Solutions'

'use client'



const inter = Inter({ subsets: ['latin'] })

export const metadata: Metadata = {
  title: 'Zeta Reticula - Smarter AI Solutions',
  description: 'Discover how Zeta Reticula transforms AI with innovative, efficient technology.',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className={inter.className}>{children}</body>
    </html>
  )
}

export const dynamic = 'force-dynamic' // This ensures the layout is always re-rendered on each request
export const revalidate = 0 // Disable static generation caching
export const fetchCache = 'force-no-store' // Disable caching for this layout
