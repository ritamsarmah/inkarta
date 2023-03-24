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
                
                PhotosPicker(selection: $viewModel.photosPickerItem, matching: .images, photoLibrary: .shared()) {
                    Image(systemName: "plus")
                }
            }
            .sheet(isPresented: $viewModel.isShowingUploadSheet, content: {
                UploadView(viewModel: .init(imageSelection: viewModel.photosPickerItem!))
            })
        }
        .alert("Error", isPresented: $viewModel.isShowingErrorAlert, actions: {
            Button("OK", role: .cancel) {
                viewModel.errorMessage = nil
            }
        }, message: {
            Text(viewModel.errorMessage ?? "")
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
