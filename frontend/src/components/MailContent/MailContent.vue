<template>
  <div class="flex flex-col grow">
    <div
      v-if="viewMode === 'rendered' && email.body_html"
      class="flex flex-col grow max-w-full"
    >
      <div class="flex flex-row border-b border-gray-200 items-center gap-4">
        <ZoomControls v-model="mailLayoutStore.mailContentZoom" />
        <Toggle
          v-model="mailLayoutStore.blockExternalRequests"
          label="Block external requests"
        />
        <div class="flex flex-row gap-1">
          <button
            class="btn btn--small"
            :class="
              mailLayoutStore.screenSize === ScreenSize.Mobile
                ? 'bg-primary text-white border-primary'
                : ''
            "
            title="Mobile (375px)"
            @click="mailLayoutStore.screenSize = ScreenSize.Mobile"
          >
            Mobile
          </button>
          <button
            class="btn btn--small"
            :class="
              mailLayoutStore.screenSize === ScreenSize.Tablet
                ? 'bg-primary text-white border-primary'
                : ''
            "
            title="Tablet (768px)"
            @click="mailLayoutStore.screenSize = ScreenSize.Tablet"
          >
            Tablet
          </button>
          <button
            class="btn btn--small"
            :class="
              mailLayoutStore.screenSize === ScreenSize.Desktop
                ? 'bg-primary text-white border-primary'
                : ''
            "
            title="Desktop (1024px)"
            @click="mailLayoutStore.screenSize = ScreenSize.Desktop"
          >
            Desktop
          </button>
        </div>
      </div>

      <div
        v-if="loadingRendered"
        class="flex items-center justify-center h-full"
      >
        <div class="text-gray-500">Loading email content...</div>
      </div>
      <div
        v-else
        :class="[
          'w-full overflow-auto py-1',
          mailLayoutStore.screenSize === ScreenSize.Mobile ? '' : 'h-full',
          mailLayoutStore.screenSize ? 'flex justify-center' : '',
        ]"
      >
        <div
          :class="[
            deviceMockupClass,
            mailLayoutStore.screenSize === ScreenSize.Mobile
              ? ''
              : 'h-full w-full',
          ]"
          :style="iframeContainerStyle"
        >
          <iframe
            :key="renderedUrl"
            :src="renderedUrl"
            :class="[
              'border-0 bg-white',
              mailLayoutStore.screenSize === ScreenSize.Mobile
                ? 'rounded-4xl w-full h-full'
                : 'h-full w-full',
              mailLayoutStore.screenSize === ScreenSize.Tablet
                ? 'rounded-lg'
                : '',
            ]"
            :style="{ zoom: mailLayoutStore.mailContentZoom }"
          ></iframe>
        </div>
      </div>
    </div>

    <div
      v-if="viewMode === 'text' && email.body_text"
      class="grow whitespace-pre-wrap p-4"
    >
      {{ email.body_text }}
    </div>

    <div v-if="viewMode === 'html' && email.body_html" class="p-4 grow">
      <CodeViewer :content="email.body_html" language="xml" />
    </div>

    <div v-if="viewMode === 'raw'" class="p-4 grow">
      <div v-if="loadingRaw" class="text-gray-500">Loading raw email...</div>
      <CodeViewer v-else :content="rawContent" />
    </div>

    <MailHeaders v-if="viewMode === 'headers'" :email="email" />
  </div>
</template>

<script setup lang="ts">
  import { ref, watch, computed } from 'vue'
  import type { EmailRecord } from '@/types/email'
  import CodeViewer from '@/components/shared/CodeViewer/CodeViewer.vue'
  import { apiClient } from '@/api/client'
  import type { ViewMode } from './MailView.vue'
  import { useMailLayoutStore, ScreenSize } from '@/stores/MailLayout'
  import ZoomControls from './ZoomControls.vue'
  import MailHeaders from './MailHeaders.vue'
  import Toggle from '@/components/shared/Toggle/Toggle.vue'

  const props = defineProps<{
    email: EmailRecord
    viewMode: ViewMode
  }>()

  const mailLayoutStore = useMailLayoutStore()
  const rawContent = ref<string>('')
  const loadingRaw = ref(false)
  const loadingRendered = ref(false)

  // Computed property for iframe container style based on screen size
  const iframeContainerStyle = computed(() => {
    const screenSize = mailLayoutStore.screenSize
    if (!screenSize) {
      return {}
    }
    const maxWidths: Record<ScreenSize, string> = {
      [ScreenSize.Mobile]: '375px',
      [ScreenSize.Tablet]: '768px',
      [ScreenSize.Desktop]: '1024px',
    }
    const style: Record<string, string> = {
      maxWidth: maxWidths[screenSize],
    }
    if (screenSize === ScreenSize.Mobile) {
      style.width = '375px'
      style.aspectRatio = '9/18'
      style.height = 'auto'
      style.flexShrink = '0'
    }
    return style
  })

  // Computed property for device mockup styling
  const deviceMockupClass = computed(() => {
    const screenSize = mailLayoutStore.screenSize
    if (screenSize === ScreenSize.Mobile) {
      return 'bg-gray-900 rounded-[2.5rem] p-2 shadow-2xl border-4 border-gray-800'
    } else if (screenSize === ScreenSize.Tablet) {
      return 'bg-gray-900 rounded-xl p-3 shadow-2xl border-4 border-gray-800'
    }
    return ''
  })

  // Computed property for iframe URL that updates when blocking state changes
  const renderedUrl = computed(() => {
    if (!props.email.body_html) return ''
    const blockExternal = mailLayoutStore.blockExternalRequests
    return `/api/emails/${props.email.id}/rendered?block_external_requests=${blockExternal}`
  })

  const loadRawEmail = async () => {
    loadingRaw.value = true
    try {
      rawContent.value = await apiClient.getRawEmail(props.email.id)
    } catch (err) {
      console.error('Failed to load raw email:', err)
      rawContent.value = 'Failed to load raw email content'
    } finally {
      loadingRaw.value = false
    }
  }

  // Watch viewMode to fetch content when switching tabs
  watch(
    () => props.viewMode,
    (newValue: ViewMode) => {
      if (newValue === 'raw') {
        loadRawEmail()
      } else if (newValue === 'rendered' && props.email.body_html) {
        // Set loading state briefly when switching to rendered tab
        loadingRendered.value = true
        // The iframe will load the content, so we can clear loading after a short delay
        setTimeout(() => {
          loadingRendered.value = false
        }, 100)
      }
    },
    { immediate: true }
  )

  // Watch blockExternalRequests to trigger iframe reload
  watch(
    () => mailLayoutStore.blockExternalRequests,
    () => {
      // The iframe src will change, triggering a reload
      // Set loading state briefly
      if (props.viewMode === 'rendered' && props.email.body_html) {
        loadingRendered.value = true
        setTimeout(() => {
          loadingRendered.value = false
        }, 100)
      }
    }
  )
</script>
