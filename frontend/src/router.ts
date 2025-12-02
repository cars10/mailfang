import { createRouter, createWebHistory } from 'vue-router'
import MailContent from './components/MailContent/MailContent.vue'
import SettingsList from './components/settings/SettingsList.vue'
import MailLayout from './components/MailLayout/MailLayout.vue'

const routes = [
  { path: '/', redirect: '/mails/inbox' },
  {
    path: '/mails/inbox',
    component: MailLayout,
    children: [{ path: ':id', component: MailContent }],
  },
  {
    path: '/mails/unread',
    component: MailLayout,
    children: [{ path: ':id', component: MailContent }],
  },
  {
    path: '/mails/with-attachments',
    component: MailLayout,
    children: [{ path: ':id', component: MailContent }],
  },
  {
    path: '/mails/archive',
    component: MailLayout,
    children: [{ path: ':id', component: MailContent }],
  },
  { path: '/settings', component: SettingsList },
  { path: '/:pathMatch(.*)*', redirect: '/' },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
