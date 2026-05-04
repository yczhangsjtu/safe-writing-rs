import { mount } from 'svelte';
import App from './App.svelte';
import './styles/global.css';
import '@fontsource/material-icons';

const app = mount(App, {
  target: document.getElementById('app')!,
});

export default app;