<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import * as pdfjsLib from 'pdfjs-dist'

// Configure PDF.js worker
pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  'pdfjs-dist/build/pdf.worker.min.mjs',
  import.meta.url
).toString()

const props = defineProps<{
  pdfData: string  // Base64 encoded PDF data
  initialPage?: number
  initialZoom?: number
}>()

const emit = defineEmits<{
  pageChange: [page: number, total: number]
  zoomChange: [zoom: number]
  documentLoaded: [totalPages: number]
}>()

const containerRef = ref<HTMLDivElement | null>(null)
const canvasContainerRef = ref<HTMLDivElement | null>(null)
const currentPage = ref(1)
const totalPages = ref(0)
const zoomLevel = ref(1.0)
const rendering = ref(false)

let pdfDoc: pdfjsLib.PDFDocumentProxy | null = null

// Initialize zoom from props
onMounted(() => {
  if (props.initialPage) currentPage.value = props.initialPage
  if (props.initialZoom) zoomLevel.value = props.initialZoom
})

// Watch for pdfData changes
watch(() => props.pdfData, async (newData) => {
  if (newData) {
    await loadDocument(newData)
  }
}, { immediate: true })

async function loadDocument(base64Data: string) {
  try {
    // Clean up previous document
    if (pdfDoc) {
      pdfDoc.destroy()
      pdfDoc = null
    }

    // Decode base64 to Uint8Array
    const binaryString = atob(base64Data)
    const bytes = new Uint8Array(binaryString.length)
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i)
    }

    const loadingTask = pdfjsLib.getDocument({ data: bytes })
    pdfDoc = await loadingTask.promise
    totalPages.value = pdfDoc.numPages

    emit('documentLoaded', pdfDoc.numPages)

    // Render the current page (or initial page)
    const targetPage = Math.min(props.initialPage || 1, pdfDoc.numPages)
    currentPage.value = targetPage
    await renderAllVisiblePages()
  } catch (error) {
    console.error('Failed to load PDF document:', error)
  }
}

async function renderAllVisiblePages() {
  if (!pdfDoc || !canvasContainerRef.value) return

  rendering.value = true
  try {
    // Clear existing canvases
    canvasContainerRef.value.innerHTML = ''

    // Determine visible page range (render a window around current page)
    const startPage = Math.max(1, currentPage.value - 2)
    const endPage = Math.min(totalPages.value, currentPage.value + 5)

    for (let pageNum = startPage; pageNum <= endPage; pageNum++) {
      const page = await pdfDoc!.getPage(pageNum)
      const viewport = page.getViewport({ scale: zoomLevel.value * window.devicePixelRatio })

      const canvas = document.createElement('canvas')
      canvas.id = `pdf-page-${pageNum}`
      canvas.className = 'pdf-page-canvas'
      canvas.width = viewport.width
      canvas.height = viewport.height
      canvas.style.width = `${viewport.width / window.devicePixelRatio}px`
      canvas.style.height = `${viewport.height / window.devicePixelRatio}px`
      canvas.dataset.pageNum = String(pageNum)

      canvasContainerRef.value.appendChild(canvas)

      const context = canvas.getContext('2d')!
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const renderTask = page.render({
        canvasContext: context,
        viewport: viewport,
      } as any)
      await renderTask.promise

      // Add page number label
      const pageLabel = document.createElement('div')
      pageLabel.className = 'pdf-page-label'
      pageLabel.textContent = `${pageNum} / ${totalPages.value}`
      canvasContainerRef.value.appendChild(pageLabel)
    }

    // Scroll to current page
    await nextTick()
    scrollToPage(currentPage.value)
  } catch (error) {
    console.error('Failed to render pages:', error)
  } finally {
    rendering.value = false
  }
}

function scrollToPage(page: number) {
  const canvas = canvasContainerRef.value?.querySelector(`#pdf-page-${page}`) as HTMLElement
  if (canvas) {
    canvas.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }
}

// Detect which page is currently visible via IntersectionObserver
let observer: IntersectionObserver | null = null

onMounted(() => {
  observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting && entry.target instanceof HTMLElement) {
          const pageNum = parseInt(entry.target.dataset.pageNum || '0', 10)
          if (pageNum > 0 && pageNum !== currentPage.value) {
            currentPage.value = pageNum
            emit('pageChange', pageNum, totalPages.value)
          }
        }
      }
    },
    { root: containerRef.value, threshold: 0.5 }
  )
})

onBeforeUnmount(() => {
  observer?.disconnect()
  if (pdfDoc) {
    pdfDoc.destroy()
    pdfDoc = null
  }
})

// Navigation
function goToPage(page: number) {
  if (page < 1 || page > totalPages.value) return
  currentPage.value = page
  scrollToPage(page)
  emit('pageChange', page, totalPages.value)
}

function prevPage() {
  goToPage(currentPage.value - 1)
}

function nextPage() {
  goToPage(currentPage.value + 1)
}

// Zoom
function setZoom(zoom: number) {
  zoomLevel.value = Math.max(0.25, Math.min(3.0, zoom))
  emit('zoomChange', zoomLevel.value)
  renderAllVisiblePages()
}

function zoomIn() {
  setZoom(zoomLevel.value + 0.15)
}

function zoomOut() {
  setZoom(zoomLevel.value - 0.15)
}

function fitWidth() {
  if (!containerRef.value || !pdfDoc) return
  // Get the first page's viewport at scale 1.0
  pdfDoc.getPage(currentPage.value).then(page => {
    const viewport = page.getViewport({ scale: 1.0 })
    const containerWidth = containerRef.value!.clientWidth - 40 // 20px padding each side
    const newZoom = containerWidth / viewport.width
    setZoom(newZoom)
  })
}

// Expose methods for parent
defineExpose({
  goToPage,
  prevPage,
  nextPage,
  zoomIn,
  zoomOut,
  fitWidth,
  setZoom,
  currentPage,
  totalPages,
  zoomLevel,
})
</script>

<template>
  <div ref="containerRef" class="pdf-viewer">
    <div v-if="rendering" class="rendering-overlay">
      <div class="spinner" />
      <span>渲染中...</span>
    </div>
    <div ref="canvasContainerRef" class="pdf-canvas-container" />
  </div>
</template>

<style scoped>
.pdf-viewer {
  flex: 1;
  overflow: auto;
  background: var(--bg-tertiary);
  position: relative;
}

.pdf-canvas-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px;
  gap: 12px;
}

.pdf-page-canvas {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  border-radius: 2px;
  background: white;
}

.pdf-page-label {
  font-size: 11px;
  color: var(--text-tertiary);
  text-align: center;
  margin-bottom: 8px;
}

.rendering-overlay {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--text-tertiary);
  font-size: 14px;
  z-index: 10;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-primary);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
