<template>
  <div class="bg-gray-100 border-r border-gray-300 shrink-0">
    <vue-resizable
      :active="['r']"
      :min-width="mailLayoutStore.sidebarCollapsed ? 100 : undefined"
      :max-width="600"
      :width="
        mailLayoutStore.sidebarCollapsed
          ? 100
          : mailLayoutStore.sidebarWidth || DEFAULT_SIDEBAR_WIDTH
      "
      class="flex flex-col h-full"
      @dblclick="handleDoubleClick"
      @resize:move="handleResizeEnd"
    >
      <div class="p-4 h-full overflow-y-auto">
        <h1 class="text-2xl font-bold mb-4 text-center">{{ headerText }}</h1>

        <div class="flex flex-col gap-2">
          <router-link
            :to="'/emails/inbox'"
            title="All emails"
            active-class="text-primary bg-gray-200"
            class="flex flex-row gap-4 items-center justify-between hover:bg-gray-200 px-2 py-1 rounded-sm text-[#222]"
          >
            <div class="flex flex-row gap-2 items-center">
              <InboxIcon class="h-4" />
              <div v-if="!mailLayoutStore.sidebarCollapsed">All emails</div>
            </div>
            <span class="text-sm text-gray-600 font-mono">
              {{ props.counts.inbox }}
            </span>
          </router-link>
        </div>

        <div class="mt-2">
          <div
            class="flex w-full flex-row items-center justify-between gap-2 rounded-sm text-[#222] h-[32px]"
          >
            <button
              class="flex flex-row shrink items-center gap-2 hover:bg-gray-200 rounded-sm px-2 py-1 cursor-pointer"
              type="button"
              @click="
                mailLayoutStore.recipientsCollapsed =
                  !mailLayoutStore.recipientsCollapsed
              "
            >
              <ChevronRightIcon
                v-if="mailLayoutStore.recipientsCollapsed"
                class="h-4"
              />
              <ChevronDownIcon v-else class="h-4" />

              <div v-if="!mailLayoutStore.sidebarCollapsed && !searchFocused">
                Inboxes
              </div>
            </button>
            <div v-if="!mailLayoutStore.sidebarCollapsed" class="ml-auto">
              <TextInput
                v-model="searchTerm"
                placeholder="Search inboxes..."
                :icon="MagnifyingGlassIcon"
                :expandable="true"
                expanded-width="100%"
                @focus="searchFocused = true"
                @blur="searchFocused = false"
              />
            </div>
          </div>

          <Transition
            enter-active-class="transition-all duration-300 ease-out"
            enter-from-class="max-h-0 opacity-0"
            enter-to-class="max-h-64 opacity-100"
            leave-active-class="transition-all duration-300 ease-in"
            leave-from-class="max-h-64 opacity-100"
            leave-to-class="max-h-0 opacity-0"
          >
            <div
              v-if="
                !mailLayoutStore.recipientsCollapsed &&
                !mailLayoutStore.sidebarCollapsed
              "
              class="mt-1 flex flex-col gap-1 ml-4 pl-2 border-l border-gray-300 overflow-hidden"
            >
              <div class="flex flex-col gap-1 max-h-64 overflow-y-auto">
                <router-link
                  v-for="recipient in filteredRecipients"
                  :key="recipient.recipient"
                  :to="`/emails/inbox/${encodeURIComponent(recipient.recipient)}`"
                  :title="recipient.recipient"
                  active-class="text-primary bg-gray-200"
                  class="flex flex-row gap-1 items-center justify-between hover:bg-gray-200 px-2 py-1 rounded-sm text-sm text-[#222]"
                >
                  <span class="truncate">{{ recipient.recipient }}</span>
                  <span class="text-xs text-gray-600 font-mono">
                    {{ recipient.count }}
                  </span>
                </router-link>
              </div>
            </div>
          </Transition>
        </div>

        <div class="flex flex-col gap-2 mt-4">
          <button
            class="btn flex flex-row items-center gap-2"
            :class="{ 'justify-center': mailLayoutStore.sidebarCollapsed }"
            :disabled="loadingDeleteAll"
            @click="handleDeleteAll"
          >
            <TrashIcon class="h-4" />
            <div v-if="!mailLayoutStore.sidebarCollapsed">Clear</div>
          </button>
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
    ChevronDownIcon,
    ChevronRightIcon,
    MagnifyingGlassIcon,
  } from '@heroicons/vue/24/outline'
  import type { EmailCounts } from '@/types/email'
  import { apiClient } from '@/api/client'
  import TextInput from '@/components/shared/TextInput/TextInput.vue'
  import VueResizable from 'vue-resizable'
  import { DEFAULT_SIDEBAR_WIDTH } from '@/stores/MailLayout'

  interface Props {
    counts: EmailCounts
  }

  const props = defineProps<Props>()

  const router = useRouter()
  const mailLayoutStore = useMailLayoutStore()
  const loadingDeleteAll = ref(false)
  const searchTerm = ref('')
  const searchFocused = ref(false)

  const headerText = computed(() =>
    mailLayoutStore.sidebarCollapsed ? 'MF' : 'MailFang'
  )

  const filteredRecipients = computed(() => {
    if (!searchTerm.value.trim()) {
      return props.counts.recipients
    }
    const term = searchTerm.value.toLowerCase()
    return props.counts.recipients.filter(recipient =>
      recipient.recipient.toLowerCase().includes(term)
    )
  })

  const handleDoubleClick = (event: MouseEvent) => {
    const target = event.target as HTMLElement
    if (target.classList.contains('resizable-r')) {
      mailLayoutStore.sidebarCollapsed = false
      mailLayoutStore.sidebarWidth = DEFAULT_SIDEBAR_WIDTH
    }
  }

  const handleResizeEnd = ({ width }: { width: number }) => {
    if (width < 160) {
      // Collapse if resized below 200px
      mailLayoutStore.sidebarCollapsed = true
      // Save the width before collapsing (in case user wants to restore it)
      if (width >= 50) {
        mailLayoutStore.sidebarWidth = width
      }
    } else {
      // Save the width and ensure it's expanded
      mailLayoutStore.sidebarWidth = width
      mailLayoutStore.sidebarCollapsed = false
    }
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
