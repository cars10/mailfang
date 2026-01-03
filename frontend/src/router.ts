import { createRouter, createWebHistory } from 'vue-router'
import EmailView from './components/EmailContent/EmailView.vue'
import FullscreenEmailView from './components/EmailContent/FullscreenEmailView.vue'
import MailLayout from './components/MailLayout/MailLayout.vue'

const routes = [
  { path: '/', redirect: '/emails/inbox' },
  {
    path: '/emails/inbox',
    component: MailLayout,
    children: [{ path: 'email/:id', component: EmailView }],
  },
  {
    path: '/emails/inbox/:recipient',
    component: MailLayout,
    children: [{ path: 'email/:id', component: EmailView }],
  },
  { path: '/emails/:id/fullscreen', component: FullscreenEmailView },
  { path: '/:pathMatch(.*)*', redirect: '/' },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
