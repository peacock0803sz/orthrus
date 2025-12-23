interface PreviewProps {
  url: string | null;
}

/** Sphinxプレビュー用iframe */
export function Preview({ url }: PreviewProps) {
  if (!url) {
    return (
      <div className="flex items-center justify-center h-full bg-gray-800 text-gray-400">
        <div className="text-center">
          <p className="text-lg mb-2">No preview available</p>
          <p className="text-sm">Select a project to start sphinx-autobuild</p>
        </div>
      </div>
    );
  }

  return (
    <iframe
      src={url}
      className="w-full h-full border-0 bg-white"
      sandbox="allow-scripts allow-same-origin"
      title="Sphinx Preview"
    />
  );
}
