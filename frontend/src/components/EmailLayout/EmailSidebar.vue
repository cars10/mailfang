<template>
  <div class="bg-app-gray-100 max-h-full">
    <vue-resizable
      :active="['r']"
      :min-width="64"
      :max-width="600"
      :width="mailLayoutStore.sidebarWidth || DEFAULT_SIDEBAR_WIDTH"
      :disable-attributes="['h']"
      class="h-dvh"
      @dblclick="handleDoubleClick"
      @resize:move="handleResizeEnd"
    >
      <div class="p-4 min-h-0 flex flex-col">
        <div class="flex flex-col min-h-0 flex-1">
          <a
            href="/"
            class="flex flex-row items-center justify-center group mb-4"
          >
            <Logo
              v-if="!sidebarCollapsed"
              class="h-6 w-6 min-w-4 mr-2 group-hover:text-primary"
            />

            <h1
              class="text-2xl text-app-gray-900 text-center flex flex-row items-center justify-center"
            >
              <template v-if="sidebarCollapsed">MF</template>
              <template v-else
                >Mail<span class="font-bold">Fang</span></template
              >
            </h1>
          </a>

          <div class="flex flex-col gap-2">
            <router-link
              :to="'/emails/inbox'"
              title="All emails"
              active-class="text-primary bg-app-gray-200"
              class="h-8 flex flex-row gap-4 items-center justify-between hover:bg-app-gray-200 px-2 py-1 rounded-sm overflow-hidden min-w-4"
            >
              <div class="flex flex-row gap-2 items-center min-w-4">
                <InboxIcon class="h-4 w-4 shrink-0" />
                <div v-if="!sidebarCollapsed" class="truncate">All emails</div>
              </div>
              <span class="text-sm text-app-gray-600 font-mono">
                {{ props.counts.inbox }}
              </span>
            </router-link>
          </div>

          <div class="flex flex-col flex-1 min-h-0">
            <EmailSidebarInboxes :counts="props.counts" />
          </div>

          <div class="my-4">
            <button
              class="btn btn--small flex flex-row items-center gap-2 w-auto"
              :class="{ 'justify-center': sidebarCollapsed }"
              :disabled="loadingDeleteAll"
              @click="handleDeleteAll"
            >
              <TrashIcon class="h-4 w-4 min-w-4" />
              <div v-if="!sidebarCollapsed">Delete all emails</div>
            </button>
          </div>
        </div>

        <div class="flex flex-col gap-2 pt-2 shrink-0">
          <div
            :class="[
              'flex',
              sidebarCollapsed ? 'justify-center' : 'justify-start',
            ]"
          >
            <ThemeToggle />
          </div>
          <a
            href="https://github.com/cars10/mailfang"
            target="_blank"
            class="hover:text-primary flex items-center gap-2 h-6"
          >
            <CodeBracketIcon class="h-4 w-4 min-w-4 min-h-4" />
            <div v-if="!sidebarCollapsed">GitHub</div>
          </a>
          <a
            href="https://mailfang.com"
            target="_blank"
            class="hover:text-primary flex items-center gap-2 h-6"
          >
            <GlobeAltIcon class="h-4 w-4 min-w-4 min-h-4" />
            <div v-if="!sidebarCollapsed">mailfang.com</div>
          </a>

          <div
            v-if="!sidebarCollapsed"
            class="text-sm text-app-gray-500 font-mono"
          >
            v{{ appVersion }}
          </div>
        </div>
      </div>
    </vue-resizable>
  </div>
</template>

<script setup lang="ts">
  import { computed, ref } from 'vue'
  import { useRouter } from 'vue-router'
  import { useMailLayoutStore } from '@/stores/MailLayout'
  import {
    InboxIcon,
    TrashIcon,
    GlobeAltIcon,
    CodeBracketIcon,
  } from '@heroicons/vue/24/outline'
  import Logo from '@/logo.svg'
  import type { EmailCounts } from '@/types/email'
  import { apiClient } from '@/api/client'
  import VueResizable from 'vue-resizable'
  import { DEFAULT_SIDEBAR_WIDTH } from '@/stores/MailLayout'
  import EmailSidebarInboxes from './EmailSidebarInboxes.vue'
  import ThemeToggle from '@/components/shared/ThemeToggle/ThemeToggle.vue'

  interface Props {
    counts: EmailCounts
  }

  const props = defineProps<Props>()

  const router = useRouter()
  const mailLayoutStore = useMailLayoutStore()
  const loadingDeleteAll = ref(false)
  const appVersion = __APP_VERSION__

  const sidebarCollapsed = computed(() => {
    return mailLayoutStore.sidebarWidth < 140
  })

  const handleDoubleClick = (event: MouseEvent) => {
    const target = event.target as HTMLElement
    if (target.classList.contains('resizable-r')) {
      mailLayoutStore.sidebarWidth = DEFAULT_SIDEBAR_WIDTH
    }
  }

  const handleResizeEnd = ({ width }: { width: number }) => {
    mailLayoutStore.sidebarWidth = width
  }

  const handleDeleteAll = async () => {
    if (loadingDeleteAll.value) return

    if (
      !confirm(
        'Are you sure you want to delete ALL emails? This action cannot be undone.'
      )
    ) {
      return
    }

    try {
      loadingDeleteAll.value = true
      await apiClient.deleteAll()
      router.push('/emails/inbox')
      window.location.reload()
    } catch (err) {
      console.error('Failed to delete all emails:', err)
    } finally {
      loadingDeleteAll.value = false
    }
  }
</script>
