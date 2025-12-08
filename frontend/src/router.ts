import { createRouter, createWebHistory } from 'vue-router'
import MailView from './components/MailContent/MailView.vue'
import FullscreenEmailView from './components/MailContent/FullscreenEmailView.vue'
import SettingsList from './components/settings/SettingsList.vue'
import MailLayout from './components/MailLayout/MailLayout.vue'

const routes = [
  { path: '/', redirect: '/mails/inbox' },
  {
    path: '/mails/inbox',
    component: MailLayout,
    children: [{ path: ':id', component: MailView }],
  },
  {
    path: '/mails/unread',
    component: MailLayout,
    children: [{ path: ':id', component: MailView }],
  },
  {
    path: '/mails/with-attachments',
    component: MailLayout,
    children: [{ path: ':id', component: MailView }],
  },
  {
    path: '/mails/archive',
    component: MailLayout,
    children: [{ path: ':id', component: MailView }],
  },
  { path: '/emails/:id/fullscreen', component: FullscreenEmailView },
  { path: '/settings', component: SettingsList },
  { path: '/:pathMatch(.*)*', redirect: '/' },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
