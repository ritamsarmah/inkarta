//
//  GalleryView.swift
//  Inkasso
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
                } else if let artworks = viewModel.artworks {
                    List {
                        ForEach(artworks) { artwork in
                            NavigationLink {
                                DetailView(viewModel: .init(artwork: artwork, parentViewModel: viewModel)
                                )
                            } label: {
                                HStack {
                                    VStack(alignment: .leading, spacing: 4) {
                                        Text(artwork.title)
                                            .font(.body)
                                        Text(artwork.artist)
                                            .font(.caption)
                                    }
                                    Spacer()
                                }
                            }
                        }
                        .onDelete { indexes in
                            Task {
                                await viewModel.delete(at: indexes)
                            }
                        }
                    }
                } else {
                    Text("No artwork available")
                        .foregroundColor(.secondary)
                }
            }
            .navigationTitle("Art")
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    EditButton()
                }
                
                ToolbarItemGroup(placement: .navigationBarTrailing) {
                    Button(action: fetch) {
                        Image(systemName: "arrow.clockwise")
                    }
                    
                    Button {
                        viewModel.isShowingFileImporter = true
                    } label: {
                        Image(systemName: "plus")
                    }
                }
            }
            .sheet(isPresented: $viewModel.isShowingUploadView, onDismiss: fetch) {
                if let url = viewModel.uploadImageURL {
                    UploadView(viewModel: .init(imageURL: url))
                }
            }
            .errorAlert(info: viewModel.errorInfo)
        }
        .fileImporter(isPresented: $viewModel.isShowingFileImporter, allowedContentTypes: [.image], onCompletion: viewModel.onImportFile)
        .onAppear(perform: fetch)
    }
    
    func fetch() {
        Task {
            viewModel.isLoading = true
            await viewModel.fetch()
            viewModel.isLoading = false
        }
    }
}

//struct GalleryView_Previews: PreviewProvider {
//    static var previews: some View {
//        GalleryView()
//    }
//}
