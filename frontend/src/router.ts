import {
  createRouter,
  createWebHistory,
  type RouteLocationNormalized,
  type NavigationGuardNext,
} from 'vue-router'
import EmailView from './components/EmailContent/EmailView.vue'
import FullscreenEmailView from './components/EmailContent/FullscreenEmailView.vue'
import EmailLayout from './components/EmailLayout/EmailLayout.vue'
import { apiClient } from './api/client'

const routes = [
  { path: '/', redirect: '/emails/inbox' },
  {
    path: '/emails/inbox',
    component: EmailLayout,
    children: [{ path: 'email/:id', component: EmailView }],
  },
  {
    path: '/emails/inbox/:recipient',
    component: EmailLayout,
    children: [{ path: 'email/:id', component: EmailView }],
  },
  {
    path: '/redirect_first_email',
    component: EmailLayout,
    beforeEnter: async (
      to: RouteLocationNormalized,
      _from: RouteLocationNormalized,
      next: NavigationGuardNext
    ) => {
      const response = await apiClient.inbox(1)
      const first = response.emails[0]
      if (first) {
        next({ path: `/emails/inbox/email/${first.id}`, query: to.query })
      } else {
        next({ path: '/emails/inbox' })
      }
    }
  },
  { path: '/emails/:id/fullscreen', component: FullscreenEmailView },
  { path: '/:pathMatch(.*)*', redirect: '/' },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
