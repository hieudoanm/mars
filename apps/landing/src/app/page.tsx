import type { FC } from 'react';
import type { NextPage } from 'next';
import Navbar from '../components/layout/Navbar';
import Hero from '../components/home/Hero';
import Bands from '../components/home/Bands';
import WhyMars from '../components/home/WhyMars';
import Workflow from '../components/home/Workflow';
import BuiltDifferent from '../components/home/BuiltDifferent';
import Tools from '../components/home/Tools';
import GetStarted from '../components/home/GetStarted';
import Footer from '../components/layout/Footer';

const Page: FC & NextPage = () => {
  return (
    <div className='relative overflow-hidden'>
      <div className='animate-drift pointer-events-none absolute top-20 right-1/12 h-130 w-130 rounded-full bg-hero-glow-1 blur-xs' />
      <div className='animate-drift-reverse pointer-events-none absolute top-150 -left-62 h-138 w-138 rounded-full bg-hero-glow-2 blur-xs' />
      <Navbar />
      <main>
        <Hero />
        <Bands />
        <WhyMars />
        <Workflow />
        <BuiltDifferent />
        <Tools />
        <GetStarted />
      </main>
      <Footer />
    </div>
  );
};

export default Page;
