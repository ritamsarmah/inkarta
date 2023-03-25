//
//  GalleryView.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/21/23.
//

import PhotosUI
import SwiftUI

struct GalleryView: View {
    
    @ObservedObject var viewModel = GalleryViewModel()
    
    var body: some View {
        NavigationView {
            Group {
                if viewModel.isLoading {
                    ProgressView()
                } else {
                    if let artworks = viewModel.artworks {
                        List(artworks, id:\.id) { artwork in
                            VStack(alignment: .leading, spacing: 4) {
                                Text(artwork.title)
                                    .font(.body)
                                Text(artwork.artist)
                                    .font(.caption)
                            }
                        }
                    } else {
                        Text("No artwork available")
                            .foregroundColor(.secondary)
                    }
                }
            }
            .navigationTitle("Art")
            .toolbar {
                Button {
                    viewModel.fetch()
                } label: {
                    Image(systemName: "arrow.clockwise")
                }
                
                Button {
                    viewModel.isShowingFileImporter = true
                } label: {
                    Image(systemName: "plus")
                }
            }
            .sheet(isPresented: $viewModel.isShowingUploadSheet, content: {
                if let url = viewModel.newImageURL {
                    UploadView(viewModel: .init(imageURL: url))
                }
            })
            .alert("Error", isPresented: $viewModel.isShowingErrorAlert, actions: {
                Button("OK", role: .cancel) {
                    viewModel.errorMessage = nil
                }
            }, message: {
                Text(viewModel.errorMessage ?? "")
            })
        }
        .fileImporter(isPresented: $viewModel.isShowingFileImporter, allowedContentTypes: [.image], onCompletion: { result in
            guard let imageURL = try? result.get() else {
                viewModel.errorMessage = "Failed to import the selected image"
                return
            }
            
            viewModel.newImageURL = imageURL
        })
        .onAppear {
            Task {
                viewModel.fetch()
            }
        }
    }
}

//struct GalleryView_Previews: PreviewProvider {
//    static var previews: some View {
//        GalleryView()
//    }
//}
